use fltk::{
    app,
    button::Button,
    frame::Frame,
    group::{Pack, PackType},
    input::IntInput,
    prelude::*,
    window::Window,
};

pub struct GotoPrompt {
    window: Window,
    paragraph_number_input: IntInput,
    ok_button: Button,
}

impl GotoPrompt {
    pub fn new() -> GotoPrompt {
        let mut goto_window = Window::default()
            .with_label("Goto Paragraph Number")
            .with_size(400, 100);

        let mut pack_horizontal_layout = Pack::default()
            .with_size(300, 30)
            .center_of_parent()
            .with_type(PackType::Horizontal);
        pack_horizontal_layout.set_spacing(20);

        Frame::default()
            .with_size(125, 0)
            .with_label("Enter Paragraph Number:");

        let paragraph_num_input = IntInput::default().with_size(100, 0);

        let ok_button = Button::default().with_size(80, 0).with_label("OK");
        pack_horizontal_layout.end();

        goto_window.end();
        goto_window.make_modal(true);

        GotoPrompt {
            window: goto_window,
            paragraph_number_input: paragraph_num_input,
            ok_button,
        }
    }

    pub fn show(&mut self, current_paragraph_num: usize) {
        let user_adjusted_paragraph_num = current_paragraph_num + 1;
        self.paragraph_number_input
            .set_value(&user_adjusted_paragraph_num.to_string());
        self.ok_button.activate();
        self.ok_button
            .take_focus()
            .expect("Could not take focus of OK Button.");

        let mut goto_window = self.window.clone();
        self.ok_button.set_callback(move |button| {
            button.deactivate();
            goto_window.hide();
        });

        self.window.show();
    }

    pub fn get_paragraph_num(&self) -> Option<usize> {
        while self.window.shown() {
            app::wait();
        }

        if self.ok_button.active() {
            return None;
        }

        let paragraph_num_choice = self
            .paragraph_number_input
            .value()
            .parse::<usize>()
            .expect("User input for paragraph number is not a integer.");

        Some(paragraph_num_choice.clamp(1, usize::MAX))
    }
}
