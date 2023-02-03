use sgui2::builders::{GuiBuilder, TabBuilder, WidgetData};
use sgui2::GuiEvent;

fn main() {
    let mut gui_builder = GuiBuilder::new("Test program");
    let mut tab = TabBuilder::new("Tab with lots of buttons");
    for x in 0..100 {
        tab.widget(WidgetData::btn(format!("Button no. {x}"), move |_, _| println!("Button {x}")));
    }
    gui_builder.tab(&mut tab);
    let mut gui = gui_builder.build();

    loop {
        if let Some(ev) = gui.tick() {
            eprintln!("{:#?}", ev);
            if ev == GuiEvent::Quit {
                return;
            }
        }
    }
}
