use fltk::{
    app,
    button::Button,
    frame::Frame,
    group::{Pack, PackType},
    input::{IntInput, Input},
    prelude::*,
    window::Window, enums::Align,
};

pub struct GotoPrompt {
    window: Window,
    paragraph_number_input: IntInput,
    ok_button: Button,
    cancel_button: Button,
}

impl GotoPrompt {
    // Used the following as reference to derive the following
    // magic numbers for ideal formatting of this particular prompt:
    // https://github.com/fltk/fltk/blob/master/src/Fl_Message.cxx#L138
    pub fn new() -> GotoPrompt {
        let mut goto_window = Window::default()
            .with_label("Goto Paragraph Number")
            .with_size(400, 75);

        let paragraph_num_input = IntInput::new(130, 10, 260, 23, "Paragraph Number:");

        let ok_button = Button::new(300, 43, 90, 23, "Go To");
        let cancel_button = Button::new(200, 43, 90, 23, "Cancel");

        goto_window.end();
        goto_window.make_modal(true);

        GotoPrompt {
            window: goto_window,
            paragraph_number_input: paragraph_num_input,
            ok_button,
            cancel_button,
        }
    }

    pub fn show(&mut self, current_paragraph_num: usize) {
        let user_adjusted_paragraph_num = current_paragraph_num + 1;
        self.paragraph_number_input
            .set_value(&user_adjusted_paragraph_num.to_string());
        self.ok_button.activate();

        let mut goto_window = self.window.clone();
        self.ok_button.set_callback(move |button| {
            button.deactivate();
            goto_window.hide();
        });

        let mut goto_window = self.window.clone();
        self.cancel_button.set_callback(move |_| {
            goto_window.hide();
        });

        self.window.show();
    }

    pub fn get_paragraph_num(&self, max_num_paragraphs: usize) -> Option<usize> {
        while self.window.shown() {
            app::wait();
        }

        if self.ok_button.active() {
            return None;
        }

        if max_num_paragraphs == 0 {
            return None;
        }

        let paragraph_num_choice = self
            .paragraph_number_input
            .value()
            .parse::<usize>()
            .unwrap_or(0);

        Some(paragraph_num_choice.clamp(1, max_num_paragraphs))
    }
}
