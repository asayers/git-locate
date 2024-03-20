use crossterm::{
    cursor::{MoveToColumn, MoveUp},
    event::{Event, KeyEvent, KeyModifiers},
    style::{Attribute, SetAttribute},
    terminal::{Clear, ClearType},
    QueueableCommand,
};
use std::{fmt::Display, io::Write, sync::Arc};

const OPTIONS_LIMIT: usize = 8;

pub fn run<T>(options: Vec<T>) -> anyhow::Result<Option<T>>
where
    T: Display + Clone + Send + Sync + 'static,
{
    let mut matcher = nucleo::Nucleo::new(nucleo::Config::DEFAULT, Arc::new(|| ()), Some(1), 1);
    for x in options {
        matcher
            .injector()
            .push(x.clone(), |cols| cols[0] = x.to_string().into());
    }

    crossterm::terminal::enable_raw_mode()?;
    let mut prompt = Prompt::default();
    let mut wtr = std::io::stderr().lock();

    let ret = loop {
        matcher.tick(10);

        let options = matcher
            .snapshot()
            .matched_items(..)
            .take(OPTIONS_LIMIT)
            .map(|x| x.data);
        print(&mut wtr, &prompt, options)?;

        if let Event::Key(key) = crossterm::event::read()? {
            match prompt.handle_event(key) {
                Action::Abort => break None,
                Action::Select => {
                    let n_options = matcher
                        .snapshot()
                        .matched_item_count()
                        .min(OPTIONS_LIMIT as u32) as isize;
                    if n_options == 0 {
                        continue;
                    }
                    let selection = prompt.selection.rem_euclid(n_options) as u32;
                    match matcher.snapshot().get_matched_item(selection) {
                        Some(x) => break Some(x.data.clone()),
                        None => continue,
                    }
                }
                Action::Continue => matcher.pattern.reparse(
                    0,
                    &prompt.input,
                    nucleo::pattern::CaseMatching::Smart,
                    nucleo::pattern::Normalization::Smart,
                    false,
                ),
            }
        }
    };

    wtr.queue(MoveToColumn(0))?
        .queue(Clear(ClearType::FromCursorDown))?;
    crossterm::terminal::disable_raw_mode()?;
    wtr.flush()?;

    Ok(ret)
}

fn print<T: Display>(
    mut wtr: impl Write,
    prompt: &Prompt,
    options: impl ExactSizeIterator<Item = T>,
) -> anyhow::Result<()> {
    wtr.queue(MoveToColumn(0))?
        .queue(Clear(ClearType::FromCursorDown))?;
    let mut n = 0;
    let selection = if options.len() == 0 {
        0
    } else {
        prompt.selection.rem_euclid(options.len() as isize) as usize
    };
    for (i, x) in options.enumerate() {
        n += 1;
        writeln!(wtr)?;
        wtr.queue(MoveToColumn(0))?;
        if i == selection {
            wtr.queue(SetAttribute(Attribute::Reverse))?;
            write!(wtr, "{}", x)?;
            wtr.queue(SetAttribute(Attribute::Reset))?;
        } else {
            write!(wtr, "{}", x)?;
        }
    }
    if n > 0 {
        wtr.queue(MoveUp(n))?;
    }
    wtr.queue(MoveToColumn(0))?;
    write!(wtr, "> {}", prompt.input)?;
    wtr.flush()?;
    Ok(())
}

#[derive(Default)]
struct Prompt {
    input: String,
    selection: isize,
}

enum Action {
    Abort,
    Select,
    Continue,
}

impl Prompt {
    fn handle_event(&mut self, event: KeyEvent) -> Action {
        match event.code {
            x if event.modifiers.contains(KeyModifiers::CONTROL) => match x {
                crossterm::event::KeyCode::Char('c') => return Action::Abort,
                _ => (),
            },
            crossterm::event::KeyCode::Char(x) => self.input.push(x),
            crossterm::event::KeyCode::Enter => return Action::Select,
            crossterm::event::KeyCode::Esc => return Action::Abort,
            crossterm::event::KeyCode::Backspace => {
                self.input.pop();
            }
            crossterm::event::KeyCode::Up => self.selection -= 1,
            crossterm::event::KeyCode::Down => self.selection += 1,
            // TODO:
            // crossterm::event::KeyCode::Left => todo!(),
            // crossterm::event::KeyCode::Right => todo!(),
            // crossterm::event::KeyCode::Home => todo!(),
            // crossterm::event::KeyCode::End => todo!(),
            // crossterm::event::KeyCode::PageUp => todo!(),
            // crossterm::event::KeyCode::PageDown => todo!(),
            // crossterm::event::KeyCode::Tab => todo!(),
            // crossterm::event::KeyCode::BackTab => todo!(),
            // crossterm::event::KeyCode::Delete => todo!(),
            // crossterm::event::KeyCode::CapsLock => todo!(),
            _ => (),
        }
        Action::Continue
    }
}
