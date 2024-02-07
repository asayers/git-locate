use crossterm::{
    cursor::{MoveToColumn, MoveUp},
    event::{Event, KeyEvent, KeyModifiers},
    terminal::{Clear, ClearType},
    QueueableCommand,
};
use std::{
    fmt::Display,
    io::{StdoutLock, Write},
    sync::Arc,
};

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

    let mut wtr = std::io::stdout().lock();
    crossterm::terminal::enable_raw_mode()?;
    let mut prompt = Prompt::default();

    let ret = loop {
        matcher.tick(10);

        let options = matcher.snapshot().matched_items(..).take(5).map(|x| x.data);
        print(&mut wtr, &prompt, options)?;

        if let Event::Key(key) = crossterm::event::read()? {
            match prompt.handle_event(key) {
                Action::Abort => break None,
                Action::Select => {
                    break matcher
                        .snapshot()
                        .get_matched_item(0)
                        .map(|x| x.data.clone())
                }
                Action::Continue => matcher.pattern.reparse(
                    0,
                    &prompt.0,
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
    wtr: &mut StdoutLock,
    prompt: &Prompt,
    options: impl Iterator<Item = T>,
) -> anyhow::Result<()> {
    wtr.queue(MoveToColumn(0))?
        .queue(Clear(ClearType::FromCursorDown))?;
    let mut n = 0;
    for x in options {
        n += 1;
        writeln!(wtr)?;
        wtr.queue(MoveToColumn(0))?;
        write!(wtr, "{}", x)?;
    }
    if n > 0 {
        wtr.queue(MoveUp(n))?;
    }
    wtr.queue(MoveToColumn(0))?;
    write!(wtr, "> {}", prompt.0)?;
    wtr.flush()?;
    Ok(())
}

#[derive(Default)]
struct Prompt(String);

enum Action {
    Abort,
    Select,
    Continue,
}

impl Prompt {
    fn handle_event(&mut self, event: KeyEvent) -> Action {
        match event.code {
            x if event.modifiers.contains(KeyModifiers::CONTROL) => match x {
                crossterm::event::KeyCode::Char('c') => Action::Abort,
                _ => Action::Continue,
            },
            crossterm::event::KeyCode::Char(x) => {
                self.0.push(x);
                Action::Continue
            }
            crossterm::event::KeyCode::Enter => Action::Select,
            crossterm::event::KeyCode::Esc => Action::Abort,
            crossterm::event::KeyCode::Backspace => {
                self.0.pop();
                Action::Continue
            }
            // TODO:
            // crossterm::event::KeyCode::Left => todo!(),
            // crossterm::event::KeyCode::Right => todo!(),
            // crossterm::event::KeyCode::Up => todo!(),
            // crossterm::event::KeyCode::Down => todo!(),
            // crossterm::event::KeyCode::Home => todo!(),
            // crossterm::event::KeyCode::End => todo!(),
            // crossterm::event::KeyCode::PageUp => todo!(),
            // crossterm::event::KeyCode::PageDown => todo!(),
            // crossterm::event::KeyCode::Tab => todo!(),
            // crossterm::event::KeyCode::BackTab => todo!(),
            // crossterm::event::KeyCode::Delete => todo!(),
            // crossterm::event::KeyCode::CapsLock => todo!(),
            _ => Action::Continue,
        }
    }
}
