use parley::{FontContext, Layout};
use std::borrow::Cow;
use vello::{
    glyph::{fello::raw::FontRef, GlyphContext},
    kurbo::{Affine, Size},
    peniko::{Brush, Color},
    SceneBuilder,
};
use xilem::{
    view::{Id, View},
    widget::{
        AccessCx, BoxConstraints, ChangeFlags, Event, EventCx, LayoutCx, LifeCycle, LifeCycleCx,
        PaintCx, UpdateCx, Widget,
    },
    Axis, MessageResult,
};

pub struct Editor {
    content: Cow<'static, str>,
}

impl Editor {
    pub fn new(content: impl Into<Cow<'static, str>>) -> Self {
        Self {
            content: content.into(),
        }
    }
}

impl<T> View<T> for Editor {
    type State = ();

    type Element = TextWidget;

    fn build(&self, cx: &mut xilem::view::Cx) -> (Id, Self::State, Self::Element) {
        let (id, element) = cx.with_new_id(|_| TextWidget::new(self.content.clone()));
        (id, (), element)
    }

    fn rebuild(
        &self,
        cx: &mut xilem::view::Cx,
        prev: &Self,
        id: &mut Id,
        state: &mut Self::State,
        element: &mut Self::Element,
    ) -> ChangeFlags {
        ChangeFlags::empty()
    }

    fn message(
        &self,
        id_path: &[Id],
        state: &mut Self::State,
        message: Box<dyn std::any::Any>,
        app_state: &mut T,
    ) -> MessageResult<()> {
        MessageResult::Nop
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct ParleyBrush(pub Brush);

impl Default for ParleyBrush {
    fn default() -> ParleyBrush {
        ParleyBrush(Brush::Solid(Color::rgb8(0, 0, 0)))
    }
}

impl parley::style::Brush for ParleyBrush {}

pub struct TextWidget {
    text: Cow<'static, str>,
    layout: Option<Layout<ParleyBrush>>,
}

impl TextWidget {
    pub fn new(text: Cow<'static, str>) -> TextWidget {
        TextWidget { text, layout: None }
    }

    pub fn set_text(&mut self, text: Cow<'static, str>) -> ChangeFlags {
        self.text = text;
        ChangeFlags::LAYOUT | ChangeFlags::PAINT
    }

    fn get_layout_mut(&mut self, font_cx: &mut FontContext) -> &mut Layout<ParleyBrush> {
        // Ensure Parley layout is initialised
        if self.layout.is_none() {
            let mut lcx = parley::LayoutContext::new();
            let mut layout_builder = lcx.ranged_builder(font_cx, &self.text, 1.0);
            layout_builder.push_default(&parley::style::StyleProperty::Brush(ParleyBrush(
                Brush::Solid(Color::rgb8(255, 255, 255)),
            )));
            self.layout = Some(layout_builder.build());
        }

        self.layout.as_mut().unwrap()
    }

    fn layout_text(&mut self, font_cx: &mut FontContext, bc: &BoxConstraints) -> Size {
        // Compute max_advance from box constraints
        let max_advance = if bc.max().width.is_finite() {
            Some(bc.max().width as f32)
        } else if bc.min().width.is_sign_negative() {
            Some(0.0)
        } else {
            None
        };

        // Layout text
        let layout = self.get_layout_mut(font_cx);
        layout.break_all_lines(max_advance, parley::layout::Alignment::Start);

        Size {
            width: layout.width() as f64,
            height: layout.height() as f64,
        }
    }
}

impl Widget for TextWidget {
    fn event(&mut self, _cx: &mut EventCx, event: &Event) {
        dbg!(event);
    }

    fn lifecycle(&mut self, _cx: &mut LifeCycleCx, _event: &LifeCycle) {}

    fn update(&mut self, cx: &mut UpdateCx) {
        // All changes potentially require layout. Note: we could be finer
        // grained, maybe color changes wouldn't.
        cx.request_layout();
    }

    fn compute_max_intrinsic(&mut self, axis: Axis, cx: &mut LayoutCx, bc: &BoxConstraints) -> f64 {
        let size = self.layout_text(cx.font_cx(), bc);
        match axis {
            Axis::Horizontal => size.width,
            Axis::Vertical => size.height,
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx, bc: &BoxConstraints) -> Size {
        cx.request_paint();
        self.layout_text(cx.font_cx(), bc)
    }

    fn paint(&mut self, _cx: &mut PaintCx, builder: &mut SceneBuilder) {
        if let Some(layout) = &self.layout {
            render_text(builder, Affine::IDENTITY, layout);
        }
    }

    fn accessibility(&mut self, cx: &mut AccessCx) {
        let mut builder = accesskit::NodeBuilder::new(accesskit::Role::StaticText);
        builder.set_value(self.text.clone());
        cx.push_node(builder);
    }
}

pub fn render_text(builder: &mut SceneBuilder, transform: Affine, layout: &Layout<ParleyBrush>) {
    let mut gcx = GlyphContext::new();
    for line in layout.lines() {
        for glyph_run in line.glyph_runs() {
            let mut x = glyph_run.offset();
            let y = glyph_run.baseline();
            let run = glyph_run.run();
            let font = run.font();
            let font_size = run.font_size();
            let font_ref = font.as_ref();
            if let Ok(font_ref) = FontRef::from_index(font_ref.data, font.index()) {
                let style = glyph_run.style();
                let vars: [(&str, f32); 0] = [];
                let mut gp = gcx.new_provider(&font_ref, None, font_size, false, vars);
                let mut brushes = [
                    ParleyBrush(Brush::Solid(Color::rgb(0., 1., 0.))),
                    ParleyBrush(Brush::Solid(Color::rgb(1., 0., 0.))),
                ];
                for (idx, glyph) in glyph_run.glyphs().enumerate() {
                    if let Some(fragment) = gp.get(glyph.id, Some(&brushes[idx & 1].0)) {
                        let gx = x + glyph.x;
                        let gy = y - glyph.y;
                        let xform = Affine::translate((gx as f64, gy as f64))
                            * Affine::scale_non_uniform(1.0, -1.0);
                        builder.append(&fragment, Some(transform * xform));
                    }
                    x += glyph.advance;
                }
            }
        }
    }
}
