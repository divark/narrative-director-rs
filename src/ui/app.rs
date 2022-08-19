use std::rc::Rc;

use fltk::{
    app::{self, App},
    button::Button,
    enums::{Align, FrameType, Shortcut},
    frame::Frame,
    group::{self, Flex},
    menu::{self, MenuItem, SysMenuBar},
    prelude::*,
    text::{self, TextBuffer, TextDisplay},
    valuator::{Scrollbar, ScrollbarType},
    window::Window,
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

pub struct ViewerWidgets {
    pub paragraph_view: TextDisplay,
    pub next_button: Rc<Button>,
    pub prev_button: Rc<Button>,

    pub progress_counter: Frame,
}

pub struct MediaTrackingWidgets {
    pub progress_bar: Scrollbar,
    pub time_progress_label: Frame,
    pub status_bar: TextDisplay,
}

//#[derive(Clone)]
pub struct MainUIWidgets {
    pub open_menu_item: MenuItem,
    pub goto_menu_item: MenuItem,
    pub preferences_menu_item: MenuItem,

    pub play_button: Button,
    pub stop_button: Button,
    pub record_button: Button,

    pub next_button: Rc<Button>,
    pub prev_button: Rc<Button>,
}

pub struct MainApplication {
    pub app: App,
    pub main_window: Window,
    pub ui_action_receiver: fltk::app::Receiver<UIActions>,
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
        UIActions::GoTo,
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

    flex_column_layout.set_size(&menu_bar, 30);

    menu_bar
}

fn create_widget_layout(
    action_broadcaster: &fltk::app::Sender<UIActions>,
    flex_column_layout: &mut Flex,
    menu_bar: &SysMenuBar,
) -> (ViewerWidgets, MediaTrackingWidgets, MainUIWidgets) {
    // Paragraph Counter widget
    let counter_text = Frame::default().with_label("0/0").with_align(Align::Center);

    flex_column_layout.set_size(&counter_text, counter_text.label_size());

    // Paragraph Viewer Widget
    let viewer_text = text::TextBuffer::default();

    let mut paragraph_viewer = TextDisplay::default();
    paragraph_viewer.set_buffer(viewer_text);

    // Text Navigation and Audio Progress
    let progress_bar = Scrollbar::default().with_type(ScrollbarType::HorizontalFill);
    flex_column_layout.set_size(&progress_bar, 30);

    let navigation_pack = Flex::default_fill().with_type(group::FlexType::Row);

    let mut previous_button = Button::default().with_label("<");
    previous_button.emit(*action_broadcaster, UIActions::Previous);

    let audio_progress_text = Frame::default()
        .with_label("00:00:00/00:00:00")
        .with_align(Align::Center);

    let mut next_button = Button::default().with_label(">");
    next_button.emit(*action_broadcaster, UIActions::Next);

    navigation_pack.end();
    flex_column_layout.set_size(&navigation_pack, 30);

    let next_button = Rc::new(next_button);
    let prev_button = Rc::new(previous_button);

    // Playback Widgets
    let playback_pack = Flex::default_fill().with_type(group::FlexType::Row);

    let mut stop_button = Button::default().with_label("Stop");
    stop_button.emit(*action_broadcaster, UIActions::Stop);

    let mut record_button = Button::default().with_label("Record");
    record_button.emit(*action_broadcaster, UIActions::Record);

    let mut play_pause_button = Button::default().with_label("Play");
    play_pause_button.emit(*action_broadcaster, UIActions::Play);

    playback_pack.end();
    flex_column_layout.set_size(&playback_pack, 30);

    // Status Bar
    let status_bar_buf = TextBuffer::default();

    let mut status_bar = TextDisplay::default();
    status_bar.set_buffer(status_bar_buf);

    flex_column_layout.set_size(&status_bar, 25);

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

pub fn create_main_application() -> MainApplication {
    let app = App::default().with_scheme(app::Scheme::Gleam);
    let (broadcaster, receiver) = fltk::app::channel::<UIActions>();

    // 1: Create UI.
    let mut main_window = Window::new(100, 100, 640, 480, "Narrative Director");

    let mut flex_column_layout = Flex::default_fill();
    flex_column_layout.set_type(group::FlexType::Column);

    let menu_bar = create_menu_bar(&broadcaster, &mut flex_column_layout);

    create_widget_layout(&broadcaster, &mut flex_column_layout, &menu_bar);

    // 2: Modify UI Properties
    main_window.make_resizable(true);
    main_window.end();
    main_window.show();

    main_window.size_range(640, 480, 0, 0);

    MainApplication {
        app,
        main_window,
        ui_action_receiver: receiver,
    }
}
