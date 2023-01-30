use fltk::{
    app,
    enums::FrameType,
    frame::Frame,
    group::{Flex, FlexType},
    image::PngImage,
    prelude::*,
    text::{TextBuffer, TextDisplay, WrapMode},
    window::Window,
};

pub struct AboutDialog {
    about_window: Window,
}

impl AboutDialog {
    pub fn new() -> AboutDialog {
        let mut about_window = Window::default()
            .with_label("About Narrative Director")
            .with_size(640, 360);

        let mut flex_column_layout = Flex::default_fill();
        flex_column_layout.set_type(FlexType::Column);

        let mut flex_row_layout = Flex::default_fill();
        flex_row_layout.set_type(FlexType::Row);

        let mut frame = Frame::default_fill().center_of_parent();
        frame.set_frame(FrameType::EngravedBox);

        let mut logo = PngImage::from_data(include_bytes!("../../../resources/images/icon.png"))
            .expect("Logo not found.");
        logo.scale(200, 200, true, true);
        frame.set_image(Some(logo));

        let mut description = TextDisplay::default();
        let mut description_text = TextBuffer::default();
        description_text.set_text("Version: 1.0.0\n\nNarrative Director is an alternative Audio/Video Recording application tailored for working on medium to large-sized projects. This tool aspires to keep editing to a minimum with the capability of playing, recording and re-recording readings in place at the paragraph level for some text piece, whether it's a script, or a novel.");
        description.set_buffer(description_text);
        description.wrap_mode(WrapMode::AtBounds, 0);

        flex_row_layout.end();
        flex_column_layout.end();

        about_window.end();
        about_window.make_modal(true);

        AboutDialog { about_window }
    }

    pub fn show(&mut self) {
        self.about_window.show();
        while self.about_window.shown() {
            app::wait();
        }
    }
}
