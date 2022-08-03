use fltk::{
    app::{self, App},
    enums::{FrameType, Shortcut},
    menu,
    prelude::*,
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
        "&File/Open...\t",
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
        "&File/Quit...\t",
        Shortcut::Ctrl | 'q',
        menu::MenuFlag::Normal,
        *action_broadcaster,
        UIActions::Quit,
    );

    // Edit Menu Options
    menu_bar.add_emit(
        "&Edit/Go To...\t",
        Shortcut::Ctrl | 'g',
        menu::MenuFlag::MenuDivider,
        *action_broadcaster,
        UIActions::GoTo,
    );

    menu_bar.add_emit(
        "&Edit/Preferences...\t",
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

pub fn create_main_application() -> MainApplication {
    let app = App::default().with_scheme(app::Scheme::Gtk);
    let (broadcaster, receiver) = fltk::app::channel::<UIActions>();

    // 1: Create UI.
    let mut main_window = Window::new(100, 100, 800, 600, "Narrative Director");

    let menu_bar = create_menu_bar(&broadcaster);

    // 2: Modify UI Properties
    main_window.make_resizable(true);
    main_window.end();
    main_window.show();

    MainApplication {
        app,
        ui_action_receiver: receiver,

        main_window,
        menu_bar,
    }
}
