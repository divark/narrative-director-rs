use fltk::{app, button::Button, input::IntInput, prelude::*, window::Window};

pub struct GotoPrompt {
    window: Window,
    paragraph_number_input: IntInput,
    goto_button: Button,
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

        let mut paragraph_num_input = IntInput::new(130, 10, 260, 23, "Paragraph Number:");
        let paragraph_input_label_offset = paragraph_num_input.label_size();
        paragraph_num_input.set_pos(
            paragraph_num_input.x() + paragraph_input_label_offset,
            paragraph_num_input.y(),
        );
        paragraph_num_input.set_size(
            paragraph_num_input.width() - paragraph_input_label_offset,
            paragraph_num_input.height(),
        );

        let ok_button = Button::new(300, 43, 90, 23, "Go To");
        let cancel_button = Button::new(200, 43, 90, 23, "Cancel");

        goto_window.end();
        goto_window.make_modal(true);

        GotoPrompt {
            window: goto_window,
            paragraph_number_input: paragraph_num_input,
            goto_button: ok_button,
            cancel_button,
        }
    }

    /// Shows the Goto Prompt with the current_paragraph_num
    /// being pre-populated in the input field + 1.
    ///
    /// Preconditions:
    /// current_paragraph_num >= 0
    ///
    /// Postconditions:
    /// Input value = current_paragraph_num + 1
    /// Goto Prompt is visible
    /// Goto Button is now active.
    pub fn show(&mut self, current_paragraph_num: usize) {
        let user_adjusted_paragraph_num = current_paragraph_num + 1;
        self.paragraph_number_input
            .set_value(&user_adjusted_paragraph_num.to_string());
        self.goto_button.activate();

        let mut goto_window = self.window.clone();
        self.goto_button.set_callback(move |button| {
            button.deactivate();
            goto_window.hide();
        });

        let mut goto_window = self.window.clone();
        self.cancel_button.set_callback(move |_| {
            goto_window.hide();
        });

        self.window.show();
    }

    /// Waits, and then returns the user chosen paragraph number,
    /// or None if canceled.
    ///
    /// Preconditions:
    /// max_num_paragraphs > input value
    ///
    /// Postconditions:
    /// 1 <= Result <= max_num_paragraphs
    pub fn get_paragraph_num(&self, max_num_paragraphs: usize) -> Option<usize> {
        while self.window.shown() {
            app::wait();
        }

        if self.goto_button.active() {
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

#[cfg(test)]
mod test {
    use fltk::prelude::{InputExt, WidgetExt};

    use super::GotoPrompt;

    const MAX_NUM_PARAGRAPHS: usize = 2;
    const MIN_NUM_PARAGRAPHS: usize = 0;

    const CURRENT_NUM_PRG_BOUNDED: usize = 1;
    const CURRENT_NUM_PRG_UNBOUNDED: usize = MAX_NUM_PARAGRAPHS + 1;

    #[test]
    fn goto_button_inactive_on_choice() {
        let mut goto_prompt = GotoPrompt::new();
        assert!(!goto_prompt.window.visible());

        goto_prompt.show(CURRENT_NUM_PRG_BOUNDED);
        assert!(goto_prompt.goto_button.active());

        goto_prompt.goto_button.do_callback();
        assert!(!goto_prompt.goto_button.active());
    }

    #[test]
    fn goto_button_active_on_no_choice() {
        let mut goto_prompt = GotoPrompt::new();
        assert!(!goto_prompt.window.visible());

        goto_prompt.show(CURRENT_NUM_PRG_BOUNDED);
        assert!(goto_prompt.goto_button.active());

        goto_prompt.cancel_button.do_callback();
        assert!(goto_prompt.goto_button.active());
    }

    #[test]
    fn goto_no_paragraphs() {
        let mut goto_prompt = GotoPrompt::new();
        assert!(!goto_prompt.window.visible());

        goto_prompt.show(0);
        assert!(goto_prompt.window.visible());
        assert_eq!(
            goto_prompt
                .paragraph_number_input
                .value()
                .parse::<usize>()
                .unwrap(),
            1
        );

        goto_prompt.goto_button.do_callback();
        assert!(!goto_prompt.window.visible());

        let goto_result = goto_prompt.get_paragraph_num(MIN_NUM_PARAGRAPHS);
        assert!(goto_result.is_none());
    }

    #[test]
    fn goto_current_with_no_paragraphs() {
        let mut goto_prompt = GotoPrompt::new();
        assert!(!goto_prompt.window.visible());

        goto_prompt.show(CURRENT_NUM_PRG_BOUNDED);
        assert!(goto_prompt.window.visible());
        assert_eq!(
            goto_prompt
                .paragraph_number_input
                .value()
                .parse::<usize>()
                .unwrap(),
            CURRENT_NUM_PRG_BOUNDED + 1
        );

        goto_prompt.goto_button.do_callback();
        assert!(!goto_prompt.window.visible());

        let goto_result = goto_prompt.get_paragraph_num(0);
        assert!(goto_result.is_none());
    }

    #[test]
    fn goto_current_within_max() {
        let mut goto_prompt = GotoPrompt::new();
        assert!(!goto_prompt.window.visible());

        goto_prompt.show(CURRENT_NUM_PRG_BOUNDED);
        assert!(goto_prompt.window.visible());
        assert_eq!(
            goto_prompt
                .paragraph_number_input
                .value()
                .parse::<usize>()
                .unwrap(),
            CURRENT_NUM_PRG_BOUNDED + 1
        );

        goto_prompt.goto_button.do_callback();
        assert!(!goto_prompt.window.visible());

        let goto_result = goto_prompt.get_paragraph_num(MAX_NUM_PARAGRAPHS);
        assert!(goto_result.is_some());

        let goto_result = goto_result.unwrap();
        assert_eq!(goto_result, CURRENT_NUM_PRG_BOUNDED + 1);
    }

    #[test]
    fn goto_current_matches_max() {
        let mut goto_prompt = GotoPrompt::new();
        assert!(!goto_prompt.window.visible());

        goto_prompt.show(CURRENT_NUM_PRG_BOUNDED - 1);
        assert!(goto_prompt.window.visible());
        assert_eq!(
            goto_prompt
                .paragraph_number_input
                .value()
                .parse::<usize>()
                .unwrap(),
            CURRENT_NUM_PRG_BOUNDED
        );

        goto_prompt.goto_button.do_callback();
        assert!(!goto_prompt.window.visible());

        let goto_result = goto_prompt.get_paragraph_num(CURRENT_NUM_PRG_BOUNDED);
        assert!(goto_result.is_some());

        let goto_result = goto_result.unwrap();
        assert_eq!(goto_result, CURRENT_NUM_PRG_BOUNDED);
    }

    #[test]
    fn goto_current_exceeds_max() {
        let mut goto_prompt = GotoPrompt::new();
        assert!(!goto_prompt.window.visible());

        goto_prompt.show(CURRENT_NUM_PRG_UNBOUNDED - 1);
        assert!(goto_prompt.window.visible());
        assert_eq!(
            goto_prompt
                .paragraph_number_input
                .value()
                .parse::<usize>()
                .unwrap(),
            CURRENT_NUM_PRG_UNBOUNDED
        );

        goto_prompt.goto_button.do_callback();
        assert!(!goto_prompt.window.visible());

        let goto_result = goto_prompt.get_paragraph_num(MAX_NUM_PARAGRAPHS);
        assert!(goto_result.is_some());

        let goto_result = goto_result.unwrap();
        assert_eq!(goto_result, MAX_NUM_PARAGRAPHS);
    }

    #[test]
    fn goto_current_below_min() {
        let mut goto_prompt = GotoPrompt::new();
        assert!(!goto_prompt.window.visible());

        goto_prompt.show(0);
        assert!(goto_prompt.window.visible());
        assert_eq!(
            goto_prompt
                .paragraph_number_input
                .value()
                .parse::<usize>()
                .unwrap(),
            1
        );

        goto_prompt.goto_button.do_callback();
        assert!(!goto_prompt.window.visible());

        let goto_result = goto_prompt.get_paragraph_num(MAX_NUM_PARAGRAPHS);
        assert!(goto_result.is_some());

        let goto_result = goto_result.unwrap();
        assert_eq!(goto_result, 1);
    }
}
