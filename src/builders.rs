use crate::{Gui, WidgetState, Widget, Tab, Focus, SelectionWindow, theme::Theme};
use sdl2::{
    ttf::{self, Font},
    render::{
        Texture,
        TextureCreator,
        BlendMode,
    },
    video::WindowContext,
};
use std::{cell::Cell, mem, time::Instant};

pub struct GuiBuilder {
    name: String,
    tabs: Vec<Option<TabBuilder>>,
}

impl GuiBuilder {
    pub fn new(name: impl ToString) -> Self {
        Self {
            name: name.to_string(),
            tabs: Vec::new(),
        }
    }
    pub fn tab(&mut self, tab: &mut TabBuilder) -> &mut Self {
        self.tabs.push(Some(mem::take(tab)));
        self
    }
    pub fn tab_separator(&mut self) -> &mut Self {
        self.tabs.push(None);
        self
    }
    pub fn build(&mut self) -> Gui {
        // make sure we run on wayland if we can
        sdl2::hint::set("SDL_VIDEODRIVER", "wayland,x11,kmsdrm");
        // sdl init
        let sdl = sdl2::init().expect("Failed to initialize SDL");
        let video = sdl.video().expect("Failed to initialize SDL video");
        let window = video.window(&self.name, 1280, 800)
            .allow_highdpi()
            .resizable()
            .build().expect("Failed to create a window");
        let mut canvas = window.into_canvas()
            .present_vsync()
            .build()
            .expect("Failed to create SDL canvas");
        canvas.set_blend_mode(BlendMode::Blend);
        let texture_creator = canvas.texture_creator();

        // get font size
        let display = canvas.window().display_index()
            .expect("Failed to get display index of the window used by sgui2");
        let (dpi, _, _) = video.display_dpi(display).expect("Failed to get DPI");
        let fontsize = (20.0 * (dpi/72.0)).clamp(10.0, 100.0) as u16;

        // init our font rendering
        let ttf = ttf::init().expect("Failed to initialize SDL_ttf");
        let font = ttf.load_font(Theme::font(), fontsize).expect("Failed to load font");
        
        let mut built_tabs = Vec::new();
        for pre_tab_maybe in self.tabs.iter_mut() {
            if let Some(pre_tab) = pre_tab_maybe.take() {
                built_tabs.push(Some(pre_tab.build(&font, &texture_creator)));
            } else {
                built_tabs.push(None);
            }
        }
        
        Gui {
            canvas,
            tabs: built_tabs,
            current_tab: 0,
            current_widget: 0,
            font_height: font.height(),
            event_pump: sdl.event_pump().unwrap(),
            window_size: (1280, 800),
            focus: Focus::TabBar,
            last_interaction: Instant::now(),
            tab_scroll: 0,
            selection: SelectionWindow::new((255, 255, 255)),
            want_widget_scroll: 0,
            widget_scroll: 0,
        }
    }
}

fn draw_text(input: &str, font: &Font, texture_creator: &TextureCreator<WindowContext>) -> Texture {
    let surface = font.render(input).blended((255, 255, 255)).expect("Failed to render text");
    let texture = texture_creator.create_texture_from_surface(&surface).expect("Failed to create texture from surface");
    texture
}

#[derive(Default)]
pub struct TabBuilder {
    name: String,
    widgets: Vec<WidgetData>,
}

impl TabBuilder {
    pub fn new(name: impl ToString) -> Self {
        TabBuilder {
            name: name.to_string(),
            widgets: Vec::new(),
        }
    }
    pub fn widget(&mut self, data: WidgetData) -> &mut Self {
        self.widgets.push(data);
        self
    }
    fn build(self, font: &Font, texture_creator: &TextureCreator<WindowContext>) -> Tab {
        let text = draw_text(&self.name, font, texture_creator);
        let mut new_widgets = Vec::new();
        for widget in self.widgets.into_iter() {
            new_widgets.push(widget.draw(font, texture_creator));
        }
        Tab {
            text,
            widgets: new_widgets,
        }
    }
}

pub struct WidgetData {
    name: String,
    callback: Option<Box<dyn Fn(&mut WidgetState, &Cell<Instant>)>>,
    w_type: WidgetState,
}

impl WidgetData {
    pub fn btn(name: impl ToString, cb: impl Fn(&mut WidgetState, &Cell<Instant>) + 'static) -> Self {
        Self {
            name: name.to_string(),
            callback: Some(Box::new(cb)),
            w_type: WidgetState::Button,
        }
    }
    pub fn toggle(name: impl ToString, cb: impl Fn(&mut WidgetState, &Cell<Instant>) + 'static, state: bool) -> Self {
        Self {
            name: name.to_string(),
            callback: Some(Box::new(cb)),
            w_type: WidgetState::Toggle(state, if state { 255 } else { 0 }),
        }
    }
    pub fn slider(name: impl ToString, cb: impl Fn(&mut WidgetState, &Cell<Instant>) + 'static, state: u8) -> Self {
        Self {
            name: name.to_string(),
            callback: Some(Box::new(cb)),
            w_type: WidgetState::Slider(state, state),
        }
    }
    fn draw(self, font: &Font, texture_creator: &TextureCreator<WindowContext>) -> Widget {
        Widget {
            text: draw_text(&self.name, font, texture_creator),
            callback: self.callback,
            // make sure that function used by users is kickstarted
            next_callback: Cell::new(Instant::now()),
            state: self.w_type,
        }
    }
}
