use fltk::{
    app::{self, App},
    button::Button,
    enums::{self, Align, FrameType, Shortcut},
    frame::{self, Frame},
    group::{self, Flex, Pack, PackType},
    menu,
    prelude::*,
    text::{self, TextDisplay},
    valuator::{Scrollbar, ScrollbarType},
    window::{DoubleWindow, Window},
};

#[derive(Copy, Clone)]
pub enum UIActions {
    Next,
    Previous,
    Play,
    Stop,
    Record,
    AudioSkip(usize),

    GoTo,
    LoadFile,
    //LoadRecent(String),
    OpenPreferences,
    About,
    Quit,
}

/*
pub struct ViewerWidgets {
    pub paragraph_view: TextDisplay,
    pub next_button: Rc<Button>,
    pub prev_button: Rc<Button>,

    pub progress_counter: Frame,
}

pub struct MediaTrackingWidgets {
    ### TODO: Insert Progress and Status Bars into GUI ###
    pub progress_bar: gtk::Scrollbar,
    pub time_progress_label: Frame,
    pub status_bar: gtk::Statusbar,
}

#[derive(Clone)]
pub struct MainUIWidgets {
    pub open_menu_item: MenuItem,
    pub play_button: Button,
    pub stop_button: Button,
    pub record_button: Button,

    pub next_button: Rc<Button>,
    pub prev_button: Rc<Button>,
}
*/
pub struct MainApplication {
    pub app: App,
    pub ui_action_receiver: fltk::app::Receiver<UIActions>,

    pub main_window: Window,
    pub menu_bar: menu::SysMenuBar,
}

fn create_menu_bar(action_broadcaster: &fltk::app::Sender<UIActions>) -> menu::SysMenuBar {
    let mut menu_bar = menu::SysMenuBar::default().with_size(800, 35);
    menu_bar.set_frame(FrameType::FlatBox);

    // File Menu Options
    menu_bar.add_emit(
        "&File/Open\t",
        Shortcut::Ctrl | 'o',
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
        Shortcut::Ctrl | 'q',
        menu::MenuFlag::Normal,
        *action_broadcaster,
        UIActions::Quit,
    );

    // Edit Menu Options
    menu_bar.add_emit(
        "&Edit/Go To\t",
        Shortcut::Ctrl | 'g',
        menu::MenuFlag::MenuDivider,
        *action_broadcaster,
        UIActions::GoTo,
    );

    menu_bar.add_emit(
        "&Edit/Preferences\t",
        Shortcut::Ctrl | ',',
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

    menu_bar
}

fn create_widget_layout(main_window: &Window) {
    let mut flex_column_layout = Flex::default_fill();
    flex_column_layout.set_type(group::FlexType::Column);

    // Paragraph Counter widget
    let mut counter_text = Frame::default().with_label("0/0").with_align(Align::Center);

    flex_column_layout.set_size(&counter_text, counter_text.label_size());

    // Paragraph Viewer Widget
    let viewer_text = text::TextBuffer::default();

    let mut paragraph_viewer = TextDisplay::default();
    paragraph_viewer.set_buffer(viewer_text);

    // Text Navigation and Audio Progress
    let progress_bar = Scrollbar::default().with_type(ScrollbarType::HorizontalFill);
    flex_column_layout.set_size(&progress_bar, 30);

    let mut navigation_pack = Flex::default_fill().with_type(group::FlexType::Row);
    let previous_button = Button::default().with_label("<");
    let audio_progress_text = Frame::default()
        .with_label("00:00:00/00:00:00")
        .with_align(Align::Center);
    let next_button = Button::default().with_label(">");
    navigation_pack.end();
    flex_column_layout.set_size(&navigation_pack, 30);

    // Playback Widgets
    let mut playback_pack = Flex::default_fill().with_type(group::FlexType::Row);
    let stop_button = Button::default().with_label("Stop");
    let record_button = Button::default().with_label("Record");
    let play_pause_button = Button::default().with_label("Play");
    playback_pack.end();
    flex_column_layout.set_size(&playback_pack, 30);

    flex_column_layout.end();
}

pub fn create_main_application() -> MainApplication {
    let app = App::default().with_scheme(app::Scheme::Gleam);
    let (broadcaster, receiver) = fltk::app::channel::<UIActions>();

    // 1: Create UI.
    let mut main_window = Window::new(100, 100, 640, 480, "Narrative Director");

    let menu_bar = create_menu_bar(&broadcaster);

    create_widget_layout(&main_window);

    // 2: Modify UI Properties
    main_window.make_resizable(true);
    main_window.end();
    main_window.show();

    main_window.size_range(640, 480, 0, 0);

    MainApplication {
        app,
        ui_action_receiver: receiver,

        main_window,
        menu_bar,
    }
}
