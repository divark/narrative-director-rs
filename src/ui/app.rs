use std::path::PathBuf;

use fltk::{
    app::{self, App},
    button::Button,
    dialog,
    enums::{Align, FrameType, Shortcut},
    frame::Frame,
    group::{self, Flex},
    image,
    menu::{self, MenuItem, SysMenuBar},
    prelude::*,
    text::{self, TextBuffer, TextDisplay, WrapMode},
    valuator::HorNiceSlider,
    window::Window,
};

use crate::{media::io::Media, sessions::session::Session, text::viewer::ParagraphViewer};

use super::dialogs::{about::AboutDialog, goto::GotoPrompt, preferences::PreferencesDialog};

#[derive(Copy, Clone)]
pub enum UIActions {
    Next,
    Previous,
    Play,
    Stop,
    Record,
    AudioSkip(usize),

    OpenGoto,
    LoadFile,
    //LoadRecent(String),
    OpenPreferences,

    About,
    Quit,
}

pub struct ViewerWidgets {
    pub paragraph_view: TextDisplay,
    pub next_button: Button,
    pub prev_button: Button,

    pub progress_counter: Button,
}

pub struct MediaTrackingWidgets {
    pub progress_bar: HorNiceSlider,
    pub time_progress_label: Frame,
    pub status_bar: TextDisplay,
}

#[derive(Clone)]
pub struct MainUIWidgets {
    pub open_menu_item: MenuItem,
    pub goto_menu_item: MenuItem,
    pub preferences_menu_item: MenuItem,

    pub play_button: Button,
    pub stop_button: Button,
    pub record_button: Button,

    pub next_button: Button,
    pub prev_button: Button,
}

pub struct MainApplication {
    pub app: App,
    pub main_window: Window,
    pub ui_action_receiver: fltk::app::Receiver<UIActions>,

    // Widgets
    pub paragraph_viewer: ParagraphViewer,
    pub media_io: Media,

    // Dialogs
    pub goto_dialog: GotoPrompt,
    pub about_dialog: AboutDialog,
    pub preferences_dialog: PreferencesDialog,

    // State
    pub session: Option<Session>,
}

impl MainApplication {
    pub fn new() -> MainApplication {
        let app = App::default().with_scheme(app::Scheme::Gleam);
        let (broadcaster, receiver) = fltk::app::channel::<UIActions>();

        // 1: Create UI.
        let mut main_window = Window::new(100, 100, 640, 480, "Narrative Director");
        main_window.emit(broadcaster, UIActions::Quit);

        let mut flex_column_layout = Flex::default_fill();
        flex_column_layout.set_type(group::FlexType::Column);

        let menu_bar = create_menu_bar(&broadcaster, &mut flex_column_layout);

        let (viewer_widgets, media_tracking_widgets, ui_widgets) =
            create_widget_layout(&broadcaster, &mut flex_column_layout, &menu_bar);

        // 2: Modify UI Properties
        main_window.make_resizable(true);
        main_window.end();
        main_window.show();

        main_window.size_range(640, 480, 0, 0);
        let window_icon =
            image::PngImage::from_data(include_bytes!("../../resources/images/icon.png"))
                .expect("Could not load program icon.");
        main_window.set_icon(Some(window_icon));
        main_window.set_icon_label("Narrative Director");

        MainApplication {
            app,
            main_window,
            ui_action_receiver: receiver,

            paragraph_viewer: ParagraphViewer::new(viewer_widgets),
            media_io: Media::new(ui_widgets, media_tracking_widgets),

            goto_dialog: GotoPrompt::new(),
            about_dialog: AboutDialog::new(),
            preferences_dialog: PreferencesDialog::new(),

            session: None,
        }
    }

    fn open(&self) -> Option<PathBuf> {
        let mut file_chooser =
            dialog::NativeFileChooser::new(dialog::NativeFileChooserType::BrowseFile);
        file_chooser.set_filter("*.txt");
        file_chooser.show();

        let filename = file_chooser.filename();

        if filename.is_file() {
            return Some(filename);
        }

        None
    }

    fn load_audio_file(&mut self) {
        let current_session = self
            .session
            .as_ref()
            .expect("A session must exist if Next messages can be processed.");

        let audio_file_location = current_session
            .project_directory()
            .join(format!("part{}.wav", self.paragraph_viewer.paragraph_num()));

        self.media_io.load(audio_file_location);
    }

    fn load_text_file(&mut self, file_location: PathBuf) {
        if let Some(session) = &mut self.session {
            session.set_paragraph_num(self.paragraph_viewer.paragraph_num());
            session.save();
        }

        let session = Session::load(file_location.clone())
            .unwrap_or_else(|| Session::new(file_location.clone()));

        self.paragraph_viewer.load_paragraphs(
            file_location,
            &session.gathering_delimiters(),
            session.gathering_amount(),
        );
        self.paragraph_viewer
            .show_paragraph_at(session.paragraph_num());

        self.session = Some(session);
    }

    pub fn run(&mut self) {
        while self.app.wait() {
            if let Some(event) = self.ui_action_receiver.recv() {
                match event {
                    UIActions::Next => {
                        self.paragraph_viewer.show_next_paragraph();
                        self.load_audio_file();
                    }
                    UIActions::Previous => {
                        self.paragraph_viewer.show_previous_paragraph();
                        self.load_audio_file();
                    }
                    UIActions::Play => {
                        let output_device = self
                            .session
                            .as_ref()
                            .expect("Session should exist on playback.")
                            .audio_output();

                        self.media_io.play(output_device);
                    }
                    UIActions::Stop => {
                        self.media_io.stop();
                    }
                    UIActions::Record => {
                        let input_device = self
                            .session
                            .as_ref()
                            .expect("Session should exist on Recording")
                            .audio_input();

                        self.media_io.record(input_device);
                    }
                    UIActions::AudioSkip(pos_secs) => self.media_io.pause_at(pos_secs),
                    UIActions::OpenGoto => {
                        self.goto_dialog.show(self.paragraph_viewer.paragraph_num());

                        if let Some(chosen_paragraph_num) = self
                            .goto_dialog
                            .get_paragraph_num(self.paragraph_viewer.num_paragraphs())
                        {
                            self.paragraph_viewer
                                .show_paragraph_at(chosen_paragraph_num - 1);
                            self.load_audio_file();
                        }
                    }
                    UIActions::LoadFile => {
                        if let Some(file_path) = self.open() {
                            self.load_text_file(file_path);
                            self.load_audio_file();
                        }
                    }
                    UIActions::OpenPreferences => {
                        // TODO: Split session into AudioPreferences, TextPreferences, and Session.
                        // That way, users can use the Preferences dialog without needing an existing
                        // session open.
                        if let Some(session) = self.session.as_mut() {
                            self.preferences_dialog.show(session);

                            self.paragraph_viewer.reload_text_with(
                                &session.gathering_delimiters(),
                                session.gathering_amount(),
                            );
                            self.load_audio_file();
                        }
                    }
                    UIActions::About => self.about_dialog.show(),
                    UIActions::Quit => {
                        if let Some(session) = &mut self.session {
                            session.set_paragraph_num(self.paragraph_viewer.paragraph_num());
                            session.save();
                        }

                        break;
                    }
                }
            }
        }
    }
}

fn create_menu_bar(
    action_broadcaster: &fltk::app::Sender<UIActions>,
    flex_column_layout: &mut Flex,
) -> menu::SysMenuBar {
    let mut menu_bar = menu::SysMenuBar::default().with_size(800, 35);
    menu_bar.set_frame(FrameType::FlatBox);

    // File Menu Options
    menu_bar.add_emit(
        "&File/Open\t",
        Shortcut::Command | 'o',
        menu::MenuFlag::Normal,
        *action_broadcaster,
        UIActions::LoadFile,
    );

    // menu_bar.add_emit(
    //     "&File/Open Recent...\t",
    //     Shortcut::Ctrl | 'r',
    //     menu::MenuFlag::Normal,
    //     *action_broadcaster,
    //     UIActions::LoadFile,
    // );

    menu_bar.add_emit(
        "&File/Quit\t",
        Shortcut::Command | 'q',
        menu::MenuFlag::Normal,
        *action_broadcaster,
        UIActions::Quit,
    );

    // Edit Menu Options
    menu_bar.add_emit(
        "&Edit/Go To\t",
        Shortcut::Command | 'g',
        menu::MenuFlag::MenuDivider,
        *action_broadcaster,
        UIActions::OpenGoto,
    );

    menu_bar.add_emit(
        "&Edit/Preferences\t",
        Shortcut::Command | ',',
        menu::MenuFlag::Normal,
        *action_broadcaster,
        UIActions::OpenPreferences,
    );

    // Help Menu Options
    menu_bar.add_emit(
        "&Help/About\t",
        Shortcut::None,
        menu::MenuFlag::Normal,
        *action_broadcaster,
        UIActions::About,
    );

    flex_column_layout.fixed(&menu_bar, 30);

    menu_bar
}

fn create_widget_layout(
    action_broadcaster: &fltk::app::Sender<UIActions>,
    flex_column_layout: &mut Flex,
    menu_bar: &SysMenuBar,
) -> (ViewerWidgets, MediaTrackingWidgets, MainUIWidgets) {
    // Paragraph Counter widget
    let mut counter_text = Button::default()
        .with_label("0/0")
        .with_align(Align::Center);
    counter_text.set_frame(FrameType::NoBox);
    counter_text.clear_visible_focus();
    counter_text.emit(*action_broadcaster, UIActions::OpenGoto);

    flex_column_layout.fixed(&counter_text, counter_text.label_size());

    // Paragraph Viewer Widget
    let viewer_text = text::TextBuffer::default();

    let mut paragraph_viewer = TextDisplay::default();
    paragraph_viewer.set_buffer(viewer_text);
    paragraph_viewer.wrap_mode(WrapMode::AtColumn, 0);

    // Text Navigation and Audio Progress
    let mut progress_bar = HorNiceSlider::default();
    progress_bar.set_bounds(0.0, 0.0);
    let broadcaster_copy = *action_broadcaster;
    progress_bar.set_callback(move |slider_location| {
        broadcaster_copy.send(UIActions::AudioSkip(slider_location.value() as usize));
    });
    flex_column_layout.fixed(&progress_bar, 30);

    let navigation_pack = Flex::default_fill().with_type(group::FlexType::Row);

    let mut prev_button = Button::default().with_label("<");
    prev_button.emit(*action_broadcaster, UIActions::Previous);
    prev_button.deactivate();

    let audio_progress_text = Frame::default()
        .with_label("00:00:00/00:00:00")
        .with_align(Align::Center);

    let mut next_button = Button::default().with_label(">");
    next_button.emit(*action_broadcaster, UIActions::Next);
    next_button.deactivate();

    navigation_pack.end();
    flex_column_layout.fixed(&navigation_pack, 30);

    // Playback Widgets
    let playback_pack = Flex::default_fill().with_type(group::FlexType::Row);

    let mut stop_button = Button::default().with_label("Stop");
    stop_button.emit(*action_broadcaster, UIActions::Stop);
    stop_button.deactivate();

    let mut record_button = Button::default().with_label("Record");
    record_button.emit(*action_broadcaster, UIActions::Record);
    record_button.deactivate();

    let mut play_pause_button = Button::default().with_label("Play");
    play_pause_button.emit(*action_broadcaster, UIActions::Play);
    play_pause_button.deactivate();

    playback_pack.end();
    flex_column_layout.fixed(&playback_pack, 30);

    // Status Bar
    let status_bar_buf = TextBuffer::default();

    let mut status_bar = TextDisplay::default();
    status_bar.set_buffer(status_bar_buf);

    flex_column_layout.fixed(&status_bar, 30);

    flex_column_layout.end();

    let ui_widgets = MainUIWidgets {
        open_menu_item: menu_bar
            .find_item("&File/Open\t")
            .expect("Could not fetch newly created Open Menu Item."),
        goto_menu_item: menu_bar
            .find_item("&Edit/Go To\t")
            .expect("Could not fetch newly created Go To Menu Item."),
        preferences_menu_item: menu_bar
            .find_item("&Edit/Preferences\t")
            .expect("Could not fetch newly created Preferences Menu Item."),

        play_button: play_pause_button,
        stop_button,
        record_button,

        next_button: next_button.clone(),
        prev_button: prev_button.clone(),
    };

    let viewer_widgets = ViewerWidgets {
        paragraph_view: paragraph_viewer,
        next_button,
        prev_button,
        progress_counter: counter_text,
    };

    let media_tracking_widgets = MediaTrackingWidgets {
        progress_bar,
        time_progress_label: audio_progress_text,
        status_bar,
    };

    (viewer_widgets, media_tracking_widgets, ui_widgets)
}
