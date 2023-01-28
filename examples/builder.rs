use sgui2::builders::{GuiBuilder, WidgetData};
use sgui2::{GuiEvent, WidgetState};

fn widget_dbg(wdg: &mut WidgetState) {
    if let WidgetState::Slider(ref mut state) = wdg {
        if *state > 170 {
            *state = 170;
        }
    }
    println!("widget_dbg {:#?}", wdg);
}

fn main() {
    let mut gui = GuiBuilder::new("Test program")
        .tab("A tab")
            .widget(WidgetData::btn("Example button", widget_dbg))
            .done()
        .tab("Second tab")
            .widget(WidgetData::btn("Button with function", widget_dbg))
            .widget(WidgetData::btn("Button with closure", |b| println!("closure {:#?}", b)))
            .widget(WidgetData::btn("Button with an empty closure", |_| {}))
            .widget(WidgetData::toggle("A toggle with callback", |b| println!("{:#?}", b), true))
            .widget(WidgetData::slider("I'm a slider!", widget_dbg, 128))
            .done()
        .tab_separator()
        .tab("Separated tab")
            .widget(WidgetData::btn("A button with callback", |b| println!("{:#?}", b)))
            .done()
        .tab("Tab with a very long name to show off scrolling")
            .done()
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
