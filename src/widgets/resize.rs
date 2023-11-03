use kludgine::figures::units::UPx;
use kludgine::figures::{Fraction, IntoUnsigned, ScreenScale, Size};

use crate::context::{AsEventContext, GraphicsContext};
use crate::styles::Dimension;
use crate::widget::{ChildWidget, MakeWidget, Widget};
use crate::ConstraintLimit;

/// A widget that resizes its contained widget to an explicit size.
#[derive(Debug)]
pub struct Resize {
    /// If present, the width to apply to the child widget.
    pub width: Option<Dimension>,
    /// If present, the height to apply to the child widget.
    pub height: Option<Dimension>,
    child: ChildWidget,
}

impl Resize {
    /// Returns a reference to the child widget.
    #[must_use]
    pub fn child(&self) -> &ChildWidget {
        &self.child
    }

    /// Resizes `child` to `size`.
    #[must_use]
    pub fn to<T>(size: Size<T>, child: impl MakeWidget) -> Self
    where
        T: Into<Dimension>,
    {
        Self {
            child: ChildWidget::new(child),
            width: Some(size.width.into()),
            height: Some(size.height.into()),
        }
    }

    /// Resizes `child`'s width to `width`.
    #[must_use]
    pub fn width(width: impl Into<Dimension>, child: impl MakeWidget) -> Self {
        Self {
            child: ChildWidget::new(child),
            width: Some(width.into()),
            height: None,
        }
    }

    /// Resizes `child`'s height to `height`.
    #[must_use]
    pub fn height(height: impl Into<Dimension>, child: impl MakeWidget) -> Self {
        Self {
            child: ChildWidget::new(child),
            width: None,
            height: Some(height.into()),
        }
    }
}

impl Widget for Resize {
    fn redraw(&mut self, context: &mut GraphicsContext<'_, '_, '_, '_, '_>) {
        let child = self.child.mounted(&mut context.as_event_context());
        context.for_other(&child).redraw();
    }

    fn measure(
        &mut self,
        available_space: Size<ConstraintLimit>,
        context: &mut GraphicsContext<'_, '_, '_, '_, '_>,
    ) -> Size<UPx> {
        if let (Some(width), Some(height)) = (self.width, self.height) {
            Size::new(
                width.into_px(context.graphics.scale()).into_unsigned(),
                height.into_px(context.graphics.scale()).into_unsigned(),
            )
        } else {
            let available_space = Size::new(
                override_constraint(available_space.width, self.width, context.graphics.scale()),
                override_constraint(
                    available_space.height,
                    self.height,
                    context.graphics.scale(),
                ),
            );
            let child = self.child.mounted(&mut context.as_event_context());
            context.for_other(&child).measure(available_space)
        }
    }
}

fn override_constraint(
    constraint: ConstraintLimit,
    explicit: Option<Dimension>,
    scale: Fraction,
) -> ConstraintLimit {
    if let Some(size) = explicit {
        ConstraintLimit::Known(size.into_px(scale).into_unsigned())
    } else {
        constraint
    }
}