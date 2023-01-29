use sgui2::builders::{GuiBuilder, TabBuilder, WidgetData};
use sgui2::{GuiEvent, WidgetState};
use std::time::{Duration, Instant};
use std::cell::Cell;

fn widget_dbg(wdg: &mut WidgetState, _: &Cell<Instant>) {
    if let WidgetState::Slider(ref mut state, ..) = wdg {
        if *state > 170 {
            *state = 170;
        }
    }
    println!("widget_dbg {:#?}", wdg);
}

fn animate(slider: &mut WidgetState, next: &Cell<Instant>) {
    if let WidgetState::Slider(ref mut state, ref mut display_state) = slider {
        *display_state = display_state.wrapping_add(5);
        *state = *display_state;
    }
    next.set(Instant::now() + Duration::from_millis(10));
}

fn main() {
    let mut gui_builder = GuiBuilder::new("Test program");
    let mut gui = gui_builder.tab(TabBuilder::new("A tab")
            .widget(WidgetData::btn("Example button", widget_dbg)))
        .tab(TabBuilder::new("Second tab")
            .widget(WidgetData::slider("Animated slider!", animate, 128))
            .widget(WidgetData::btn("Button with function", widget_dbg))
            .widget(WidgetData::btn("Button with closure", |b, _| println!("closure {:#?}", b)))
            .widget(WidgetData::btn("Button with an empty closure", |_, _| {}))
            .widget(WidgetData::toggle("A toggle with callback", |b, _| println!("{:#?}", b), true))
            .widget(WidgetData::slider("I'm a slider!", widget_dbg, 128)))
        .tab_separator()
        .tab(TabBuilder::new("Separated tab")
            .widget(WidgetData::btn("A button with callback", |b, _| println!("{:#?}", b))))
        .tab(&mut TabBuilder::new("Tab with a very long name to show off scrolling"))
        .build();

    loop {
        if let Some(ev) = gui.tick() {
            eprintln!("{:#?}", ev);
            if ev == GuiEvent::Quit {
                return;
            }
        }
    }
}
