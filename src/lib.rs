pub mod builders;
mod theme;

use theme::{Theme, SelectionStyle};
use derivative::Derivative;
use sdl2::{
    render::{
        self,
        Texture,
    },
    video::{self},
    rect::Rect,
    event::{Event, WindowEvent},
    keyboard::Keycode,
};
use std::time::{
    Instant,
};

#[derive(Debug, PartialEq, Eq)]
pub enum GuiEvent {
    Quit,
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Gui {
    #[derivative(Debug="ignore")]
    canvas: render::Canvas<video::Window>,
    #[derivative(Debug="ignore")]
    event_pump: sdl2::EventPump,
    tabs: Vec<Option<Tab>>,
    current_tab: usize,
    current_widget: usize,
    tab_offset_y: usize,
    font_height: i32,
    window_size: (u32, u32),
    focus: Focus,
    last_interaction: Instant,
    tab_scroll: u8,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ActionKey {
    None,
    Press,
    Back,
    Up,
    Down,
    Left,
    Right,
}

impl Gui {
    pub fn tick(&mut self) -> Option<GuiEvent> {
        self.canvas.set_viewport(None);
        self.canvas.clear();

        let mut action = ActionKey::None;

        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit{..} => return Some(GuiEvent::Quit),
                Event::Window{win_event, ..} => match win_event {
                    WindowEvent::SizeChanged(..) => {
                        let tmp = self.canvas.viewport();
                        self.window_size = (tmp.width(), tmp.height());
                    },
                    _ => continue,
                },
                Event::KeyDown{keycode, repeat, ..} => {
                    if repeat {
                        continue;
                    }
                    if let Some(code) = keycode {
                        match code {
                            Keycode::Return => action = ActionKey::Press,
                            Keycode::Escape => action = ActionKey::Back,
                            Keycode::Up     => action = ActionKey::Up,
                            Keycode::Down   => action = ActionKey::Down,
                            Keycode::Left   => action = ActionKey::Left,
                            Keycode::Right  => action = ActionKey::Right,
                            _ => continue,
                        }
                    }
                }
                _ => continue,
            }
            break;
        }

        if action != ActionKey::None {
            self.last_interaction = Instant::now();
            self.tab_scroll = 0;
        }

        match action {
            ActionKey::Press => {
                if self.focus == Focus::TabBar {
                    let curtab = self.tabs.get(self.current_tab).unwrap().as_ref()
                        .unwrap();
                    if !curtab.widgets.is_empty() {
                        self.focus.bump_down();
                    }
                } else {
                    let curtab = self.tabs.get_mut(self.current_tab).unwrap().as_mut()
                        .expect("current_tab should always be a valid index for a tab");
                    if let Some(curwdg) = curtab.widgets.get_mut(self.current_widget) {
                        if curwdg.grabs_input() {
                            self.focus.bump_down();
                        }
                        curwdg.process_action(&action);
                    }
                }
            },
            ActionKey::Back => {
                self.focus.bump_up();
            },
            ActionKey::Up | ActionKey::Down | ActionKey::Left | ActionKey::Right => {
                match self.focus {
                    Focus::TabBar => {
                        let maybe_new_tab = match action {
                            ActionKey::Up => self.tabs.iter()
                                .enumerate()
                                .rev()
                                .skip_while(|(i, _)| *i+1 != self.current_tab)
                                .find_map(|(i, v)| v.as_ref().and(Some(i))),
                            ActionKey::Down => self.tabs.iter()
                                .enumerate()
                                .skip(self.current_tab + 1)
                                .find_map(|(i, v)| v.as_ref().and(Some(i))),
                            _ => None,
                        };
                        if let Some(new_tab) = maybe_new_tab {
                            self.current_tab = new_tab;
                            self.current_widget = 0;
                        }
                    },
                    Focus::Widgets => {
                        let diff: i32 = match action {
                            ActionKey::Up => -1,
                            ActionKey::Down => 1,
                            _ => 0,
                        };
                        let curtab = self.tabs.get(self.current_tab).unwrap().as_ref().unwrap();
                        let new = (self.current_widget as i32 + diff).clamp(0, i32::MAX) as usize;
                        if curtab.widgets.get(new).is_some() {
                            self.current_widget = new;
                        }
                    },
                    Focus::WidgetSingle => {
                        let curtab = self.tabs.get_mut(self.current_tab).unwrap().as_mut().unwrap();
                        if let Some(curwdg) = curtab.widgets.get_mut(self.current_widget) {
                            if curwdg.grabs_input() {
                                curwdg.process_action(&action);
                            }
                        }
                    },
                }
            }
            ActionKey::None => (),
        }

        if self.last_interaction.elapsed() > Theme::idle_timeout() {
            // idle animations
            let reset;
            (self.tab_scroll, reset) = self.tab_scroll.overflowing_add(1);
            if reset {
                self.last_interaction = Instant::now();
            }
        }

        let (width, height) = self.window_size;
        
        let one_panel: bool = height > width || width < 641;
        let left_panel;
        let right_panel;
        if one_panel {
            match self.focus {
                Focus::TabBar => {
                    left_panel = Some(self.canvas.viewport());
                    right_panel = None;
                },
                Focus::Widgets | Focus::WidgetSingle => {
                    left_panel = None;
                    right_panel = Some(self.canvas.viewport());
                }
            }
        } else {
            // 20:80 ratio, TODO: make this smarter
            let sep = width / 5; // separator
            let r_width = width - sep;
            let l_width = width - r_width;
            left_panel = Some(Rect::new(0, 0, l_width, height));
            right_panel = Some(Rect::new(sep as i32, 0, r_width, height));
        }

        if let Some(left) = left_panel {
            self.canvas.set_draw_color(Theme::bg_tabs());
            self.canvas.fill_rect(left).expect("Failed to clear left side");
            self.canvas.set_viewport(left);
            let mut y_pos = 0;
            let pad = (Theme::padding() as i32 * self.font_height)  / 100;
            for (i, t) in self.tabs.iter().enumerate() {
                y_pos += pad;
                if let Some(tab) = t {
                    let q = tab.text.query();
                    let mut out_rect = Rect::new(pad, y_pos, q.width, q.height);
                    if i == self.current_tab && self.focus == Focus::TabBar {
                        if q.width > left.width() {
                            let scroll_max = q.width - left.width() + pad as u32;
                            let scroll = (self.tab_scroll.clamp(0, 128) as f32)/128.0 * scroll_max as f32;
                            out_rect.set_x(pad + (-1 * scroll as i32));
                        }
                        match Theme::selection_style() {
                            SelectionStyle::Outline(r, g, b) => {
                                let w = left.width() - pad as u32;
                                let h = q.height as u32 + pad as u32;
                                self.canvas.set_draw_color((r, g, b));
                                self.canvas.draw_rect(Rect::new(pad/2, y_pos - pad/2, w, h))
                                    .expect("Failed to draw selection outline");
                            }
                            _ => todo!(),
                        }
                    }
                    self.canvas.copy(&tab.text, None, out_rect)
                        .expect("Failed to draw texture of a widget");
                    y_pos += q.height as i32;
                }
                y_pos += pad;
            }
        }

        if let Some(right) = right_panel {
            self.canvas.set_draw_color(Theme::bg_widgets());
            self.canvas.fill_rect(right).expect("Failed to clear right side");
            self.canvas.set_viewport(right);
            let offset = if let Some(left) = left_panel { left.width() as i32 } else { 0 };
            let curtab = self.tabs.get(self.current_tab).unwrap().as_ref()
                .expect("current_tab should always be a valid index for a tab");

            let mut y_pos = 0;
            let pad = (Theme::padding() as i32 * self.font_height)  / 100;
            self.canvas.set_draw_color(Theme::fg_widgets());
            for (i, widget) in curtab.widgets.iter().enumerate() {
                y_pos += pad as i32;

                let old_viewport = self.canvas.viewport();
                let tmp_viewport = Rect::new(offset + pad, y_pos, right.width(), widget.height());

                self.canvas.set_viewport(tmp_viewport);
                widget.draw(&mut self.canvas);
                self.canvas.set_viewport(old_viewport);

                if i == self.current_widget && self.focus != Focus::TabBar {
                    match Theme::selection_style() {
                        SelectionStyle::Outline(r, g, b) => {
                            let outline = Rect::new(0, y_pos, right.width(), widget.height());
                            self.canvas.set_draw_color((r, g, b));
                            self.canvas.draw_rect(outline).unwrap();
                            self.canvas.set_draw_color(Theme::fg_widgets());
                        },
                        _ => todo!(),
                    }
                }


                y_pos += widget.height() as i32;
                y_pos += pad as i32;
            }
            self.canvas.set_draw_color(Theme::bg_widgets());
        }

        self.canvas.present();
        None
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Focus {
    TabBar,
    Widgets,
    WidgetSingle,
}

impl Focus {
    fn bump_up(&mut self) {
        *self = match self {
            Focus::TabBar => Focus::TabBar,
            Focus::Widgets => Focus::TabBar,
            Focus::WidgetSingle => Focus::Widgets,
        }
    }
    fn bump_down(&mut self) {
        *self = match self {
            Focus::TabBar => Focus::Widgets,
            Focus::Widgets => Focus::WidgetSingle,
            Focus::WidgetSingle => Focus::Widgets,
        }
    }
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Tab {
    #[derivative(Debug="ignore")]
    text: Texture,
    widgets: Vec<Widget>,
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Widget {
    #[derivative(Debug="ignore")]
    text: Texture,
    state: WidgetState,
    #[derivative(Debug="ignore")]
    callback: Option<Box<dyn Fn(&mut WidgetState)>>,
}

impl Widget {
    fn height(&self) -> u32 {
        let query = self.text.query();
        query.height
    }
    fn draw(&self, canvas: &mut render::Canvas<video::Window>) {
        let query = self.text.query();
        let text_rect = Rect::new(0, 0, query.width, query.height);
        canvas.copy(&self.text, None, text_rect).expect("Failed to draw a widget");
        
        let bounds = canvas.viewport();
        let margin = bounds.height()/4;
        let box_size = bounds.height() - margin*2;
        match self.state {
            WidgetState::Toggle(state) => {
                let textbox_rect = Rect::new((bounds.width() - (margin*2 + box_size)) as i32, margin as i32, box_size, box_size);
                if state {
                    canvas.fill_rect(textbox_rect).expect("Failed to draw toggle widget");
                } else {
                    canvas.draw_rect(textbox_rect).expect("Failed to draw toggle widget");
                }
            },
            WidgetState::Slider(state) => {
                // try not overlapping text
                let whole_width = if query.width > bounds.width()/2 {
                    bounds.width() - query.width - margin*4
                } else {
                    bounds.width()/2 - margin*4
                };
                let x_pos = bounds.width() - whole_width - margin*2;
                let state_width = state as f32 / u8::MAX as f32 * whole_width as f32;

                let rect = Rect::new(x_pos as i32, margin as i32, whole_width, box_size);
                let state_rect = Rect::new(x_pos as i32, margin as i32, state_width as u32, box_size);

                canvas.draw_rect(rect).expect("Failed to draw slider widget");
                canvas.fill_rect(state_rect).expect("Failed to draw slider widget");
            },
            WidgetState::Button => (),
        }
    }
    fn grabs_input(&self) -> bool {
        match self.state {
            WidgetState::Button | WidgetState::Toggle(_) => false,
            WidgetState::Slider(_) => true,
        }
    }
    fn process_action(&mut self, code: &ActionKey) {
        let mut fire_callback = false;
        match self.state {
            WidgetState::Button => {
                if *code == ActionKey::Press {
                    fire_callback = true;
                }
            }
            WidgetState::Toggle(ref mut state) => {
                if *code == ActionKey::Press {
                    *state = !*state;
                    fire_callback = true;
                }
            },
            WidgetState::Slider(ref mut state) => {
                match code {
                    ActionKey::Left => {
                        *state = state.saturating_sub(12);
                        fire_callback = true;
                    },
                    ActionKey::Right => {
                        *state = state.saturating_add(12);
                        fire_callback = true;
                    },
                    _ => (),
                }
            }
        }
        if fire_callback {
            if let Some(cb) = &self.callback {
                cb(&mut self.state);
            }
        }
    }
    #[allow(dead_code, unused_variables)]
    fn process_pointer(&mut self, prev: Option<(u32, u32)>, new: (u32, u32)) {
        todo!("pointer/touch support");
    }
}

#[derive(Debug)]
pub enum WidgetState {
    Button,
    Toggle(bool),
    Slider(u8),
}
