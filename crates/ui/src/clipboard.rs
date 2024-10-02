use std::{cell::RefCell, rc::Rc, time::Duration};

use gpui::{
    prelude::FluentBuilder, AnyElement, ClipboardItem, Element, ElementId, GlobalElementId,
    IntoElement, LayoutId, ParentElement, SharedString, Styled, WindowContext,
};

use crate::{
    button::{Button, ButtonStyled as _},
    h_flex, IconName, Sizable as _,
};

pub struct Clipboard {
    id: ElementId,
    value: SharedString,
    content_builder: Option<Box<dyn Fn(&mut WindowContext) -> AnyElement>>,
    copied_callback: Option<Rc<dyn Fn(SharedString, &mut WindowContext)>>,
}

impl Clipboard {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            value: "".into(),
            content_builder: None,
            copied_callback: None,
        }
    }

    pub fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.value = value.into();
        self
    }

    pub fn content<E, F>(mut self, element_builder: F) -> Self
    where
        E: IntoElement,
        F: Fn(&mut WindowContext) -> E + 'static,
    {
        self.content_builder = Some(Box::new(move |cx| element_builder(cx).into_any_element()));
        self
    }

    pub fn on_copied<F>(mut self, handler: F) -> Self
    where
        F: Fn(SharedString, &mut WindowContext) + 'static,
    {
        self.copied_callback = Some(Rc::new(handler));
        self
    }
}

impl IntoElement for Clipboard {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

#[derive(Default)]
pub struct ClipboardState {
    copied: Rc<RefCell<bool>>,
}

impl Element for Clipboard {
    type RequestLayoutState = AnyElement;

    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone())
    }

    fn request_layout(
        &mut self,
        global_id: Option<&GlobalElementId>,
        cx: &mut WindowContext,
    ) -> (LayoutId, Self::RequestLayoutState) {
        cx.with_element_state::<ClipboardState, _>(global_id.unwrap(), |state, cx| {
            let state = state.unwrap_or_default();

            let content_element = self
                .content_builder
                .as_ref()
                .map(|builder| builder(cx).into_any_element());
            let value = self.value.clone();
            let clipboard_id = self.id.clone();
            let copied_callback = self.copied_callback.as_ref().map(|c| c.clone());
            let copied = state.copied.clone();
            let copide_value = *copied.borrow();

            let mut element = h_flex()
                .gap_1()
                .items_center()
                .when_some(content_element, |this, element| this.child(element))
                .child(
                    Button::new(clipboard_id)
                        .icon(if copide_value {
                            IconName::Check
                        } else {
                            IconName::Copy
                        })
                        .ghost()
                        .xsmall()
                        .when(!copide_value, |this| {
                            this.on_click(move |_, cx| {
                                cx.stop_propagation();
                                cx.write_to_clipboard(ClipboardItem::new_string(value.to_string()));
                                *copied.borrow_mut() = true;

                                let copied = copied.clone();
                                cx.spawn(|cx| async move {
                                    cx.background_executor().timer(Duration::from_secs(2)).await;

                                    *copied.borrow_mut() = false;
                                })
                                .detach();

                                if let Some(callback) = &copied_callback {
                                    callback(value.clone(), cx);
                                }
                            })
                        }),
                )
                .into_any_element();

            ((element.request_layout(cx), element), state)
        })
    }

    fn prepaint(
        &mut self,
        _: Option<&gpui::GlobalElementId>,
        _: gpui::Bounds<gpui::Pixels>,
        element: &mut Self::RequestLayoutState,
        cx: &mut WindowContext,
    ) {
        element.prepaint(cx);
    }

    fn paint(
        &mut self,
        _: Option<&gpui::GlobalElementId>,
        _: gpui::Bounds<gpui::Pixels>,
        element: &mut Self::RequestLayoutState,
        _: &mut Self::PrepaintState,
        cx: &mut WindowContext,
    ) {
        element.paint(cx)
    }
}
