use fltk::prelude::WidgetExt;

pub fn shift_right_by_label<T>(choice: &mut T)
where
    T: WidgetExt,
{
    let choice_label_offset = choice.label_size();
    choice.set_pos(choice.x() + choice_label_offset, choice.y());
    choice.set_size(choice.width() - choice_label_offset, choice.height());
}

// TODO: Vertical Stacker builder type to reduce magic number usage.
