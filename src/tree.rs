use std::fmt::Debug;
use std::mem;
use std::sync::{Arc, Mutex, PoisonError};

use alot::{LotId, Lots};
use kludgine::figures::units::Px;
use kludgine::figures::{Point, Rect};

use crate::styles::{ComponentDefaultvalue, Styles};
use crate::widget::{ManagedWidget, WidgetInstance};

#[derive(Clone, Default)]
pub struct Tree {
    data: Arc<Mutex<TreeData>>,
}

impl Tree {
    pub fn push_boxed(
        &self,
        widget: WidgetInstance,
        parent: Option<&ManagedWidget>,
    ) -> ManagedWidget {
        let mut data = self.data.lock().map_or_else(PoisonError::into_inner, |g| g);
        let id = WidgetId(data.nodes.push(Node {
            widget: widget.clone(),
            children: Vec::new(),
            parent: parent.map(|parent| parent.id),
            last_rendered_location: None,
            styles: None,
        }));
        if let Some(parent) = parent {
            let parent = &mut data.nodes[parent.id.0];
            parent.children.push(id);
        }
        ManagedWidget {
            id,
            widget,
            tree: self.clone(),
        }
    }

    pub fn remove_child(&self, child: &ManagedWidget, parent: &ManagedWidget) {
        let mut data = self.data.lock().map_or_else(PoisonError::into_inner, |g| g);
        data.remove_child(child.id, parent.id);
    }

    pub(crate) fn note_rendered_rect(&self, widget: WidgetId, rect: Rect<Px>) {
        let mut data = self.data.lock().map_or_else(PoisonError::into_inner, |g| g);
        rect.extents();
        data.nodes[widget.0].last_rendered_location = Some(rect);
        data.render_order.push(widget);
    }

    pub(crate) fn last_rendered_at(&self, widget: WidgetId) -> Option<Rect<Px>> {
        let data = self.data.lock().map_or_else(PoisonError::into_inner, |g| g);
        data.nodes[widget.0].last_rendered_location
    }

    pub(crate) fn reset_render_order(&self) {
        let mut data = self.data.lock().map_or_else(PoisonError::into_inner, |g| g);
        data.render_order.clear();
    }

    pub(crate) fn hover(&self, new_hover: Option<&ManagedWidget>) -> Result<HoverResults, ()> {
        let mut data = self.data.lock().map_or_else(PoisonError::into_inner, |g| g);
        let mut hovered = new_hover
            .map(|new_hover| data.widget_hierarchy(new_hover.id, self))
            .unwrap_or_default();
        match data.update_tracked_widget(new_hover, self, |data| &mut data.hover)? {
            Some(old_hover) => {
                let mut old_hovered = data.widget_hierarchy(old_hover.id, self);
                // For any widgets that were shared, remove them, as they don't
                // need to have their events fired again.
                while !old_hovered.is_empty() && old_hovered.get(0) == hovered.get(0) {
                    old_hovered.remove(0);
                    hovered.remove(0);
                }

                Ok(HoverResults {
                    unhovered: old_hovered,
                    hovered,
                })
            }
            None => Ok(HoverResults {
                unhovered: Vec::new(),
                hovered,
            }),
        }
    }

    pub fn focus(&self, new_focus: Option<&ManagedWidget>) -> Result<Option<ManagedWidget>, ()> {
        let mut data = self.data.lock().map_or_else(PoisonError::into_inner, |g| g);
        data.update_tracked_widget(new_focus, self, |data| &mut data.focus)
    }

    pub fn activate(
        &self,
        new_active: Option<&ManagedWidget>,
    ) -> Result<Option<ManagedWidget>, ()> {
        let mut data = self.data.lock().map_or_else(PoisonError::into_inner, |g| g);
        data.update_tracked_widget(new_active, self, |data| &mut data.active)
    }

    pub fn widget(&self, id: WidgetId) -> ManagedWidget {
        let data = self.data.lock().map_or_else(PoisonError::into_inner, |g| g);
        data.widget(id, self)
    }

    pub fn active_widget(&self) -> Option<WidgetId> {
        self.data
            .lock()
            .map_or_else(PoisonError::into_inner, |g| g)
            .active
    }

    pub fn hovered_widget(&self) -> Option<WidgetId> {
        self.data
            .lock()
            .map_or_else(PoisonError::into_inner, |g| g)
            .hover
    }

    pub fn is_hovered(&self, id: WidgetId) -> bool {
        let data = self.data.lock().map_or_else(PoisonError::into_inner, |g| g);
        let mut search = data.hover;
        while let Some(hovered) = search {
            if hovered == id {
                return true;
            }
            search = data.nodes[hovered.0].parent;
        }

        false
    }

    pub fn focused_widget(&self) -> Option<WidgetId> {
        self.data
            .lock()
            .map_or_else(PoisonError::into_inner, |g| g)
            .focus
    }

    pub(crate) fn widgets_at_point(&self, point: Point<Px>) -> Vec<ManagedWidget> {
        let data = self.data.lock().map_or_else(PoisonError::into_inner, |g| g);
        let mut hits = Vec::new();
        for id in data.render_order.iter().rev() {
            if let Some(last_rendered) = data.nodes[id.0].last_rendered_location {
                if last_rendered.contains(point) {
                    hits.push(ManagedWidget {
                        id: *id,
                        widget: data.nodes[id.0].widget.clone(),
                        tree: self.clone(),
                    });
                }
            }
        }
        hits
    }

    pub(crate) fn parent(&self, id: WidgetId) -> Option<WidgetId> {
        let data = self.data.lock().map_or_else(PoisonError::into_inner, |g| g);
        data.nodes[id.0].parent
    }

    pub(crate) fn attach_styles(&self, id: WidgetId, styles: Styles) {
        let mut data = self.data.lock().map_or_else(PoisonError::into_inner, |g| g);
        data.nodes[id.0].styles = Some(styles);
    }

    pub fn query_style(
        &self,
        perspective: &ManagedWidget,
        query: &[&dyn ComponentDefaultvalue],
    ) -> Styles {
        self.data
            .lock()
            .map_or_else(PoisonError::into_inner, |g| g)
            .query_style(perspective.id, query)
    }
}

pub(crate) struct HoverResults {
    pub unhovered: Vec<ManagedWidget>,
    pub hovered: Vec<ManagedWidget>,
}

#[derive(Default)]
struct TreeData {
    nodes: Lots<Node>,
    active: Option<WidgetId>,
    focus: Option<WidgetId>,
    hover: Option<WidgetId>,
    render_order: Vec<WidgetId>,
}

impl TreeData {
    fn widget(&self, id: WidgetId, tree: &Tree) -> ManagedWidget {
        ManagedWidget {
            id,
            widget: self.nodes[id.0].widget.clone(),
            tree: tree.clone(),
        }
    }

    fn remove_child(&mut self, child: WidgetId, parent: WidgetId) {
        let removed_node = self.nodes.remove(child.0).expect("widget already removed");
        let parent = &mut self.nodes[parent.0];
        let index = parent
            .children
            .iter()
            .enumerate()
            .find_map(|(index, c)| (*c == child).then_some(index))
            .expect("child not found in parent");
        parent.children.remove(index);
        let mut detached_nodes = removed_node.children;

        while let Some(node) = detached_nodes.pop() {
            let mut node = self.nodes.remove(node.0).expect("detached node missing");
            detached_nodes.append(&mut node.children);
        }
    }

    pub(crate) fn widget_hierarchy(&self, mut widget: WidgetId, tree: &Tree) -> Vec<ManagedWidget> {
        let mut hierarchy = Vec::new();
        loop {
            hierarchy.push(self.widget(widget, tree));
            let Some(parent) = self.nodes[widget.0].parent else {
                break;
            };
            widget = parent;
        }

        hierarchy.reverse();

        hierarchy
    }

    fn update_tracked_widget(
        &mut self,
        new_widget: Option<&ManagedWidget>,
        tree: &Tree,
        property: impl FnOnce(&mut Self) -> &mut Option<WidgetId>,
    ) -> Result<Option<ManagedWidget>, ()> {
        match (
            mem::replace(property(self), new_widget.map(|w| w.id)),
            new_widget,
        ) {
            (Some(old_widget), Some(new_widget)) if old_widget == new_widget.id => Err(()),
            (Some(old_widget), _) => Ok(Some(ManagedWidget {
                id: old_widget,
                widget: self.nodes[old_widget.0].widget.clone(),
                tree: tree.clone(),
            })),
            (None, _) => Ok(None),
        }
    }

    fn query_style(
        &self,
        mut perspective: WidgetId,
        query: &[&dyn ComponentDefaultvalue],
    ) -> Styles {
        let mut query = query.iter().map(|n| n.name()).collect::<Vec<_>>();
        let mut resolved = Styles::new();
        while !query.is_empty() {
            let node = &self.nodes[perspective.0];
            if let Some(styles) = &node.styles {
                query.retain(|name| {
                    if let Some(component) = styles.get(name) {
                        resolved.insert(name, component.clone());
                        false
                    } else {
                        true
                    }
                });
            }
            let Some(parent) = node.parent else { break };
            perspective = parent;
        }
        resolved
    }
}

pub struct Node {
    pub widget: WidgetInstance,
    pub children: Vec<WidgetId>,
    pub parent: Option<WidgetId>,
    pub last_rendered_location: Option<Rect<Px>>,
    pub styles: Option<Styles>,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct WidgetId(LotId);
