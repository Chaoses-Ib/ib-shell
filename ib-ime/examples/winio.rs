//! cargo run --example winio
#![allow(unused_must_use)]
use winio::prelude::*;

fn main() -> Result<()> {
    App::new("ib-ime.example")?.run::<MainModel>(())
}

struct MainModel {
    window: Child<Window>,
    edit: Child<Edit>,
    edit2: Child<Edit>,
}

enum MainMessage {
    Noop,
    Close,
    EditChange,
    Edit2Change,
}

impl Component for MainModel {
    type Error = Error;
    type Event = ();
    type Init<'a> = ();
    type Message = MainMessage;

    async fn init(_init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        // create & initialize the window
        let mut window = Child::<Window>::init(()).await?;

        let edit = Child::<Edit>::init(&window).await?;

        let edit2 = Child::<Edit>::init(&window).await?;
        ib_ime::hook::ImeHookConfig::default_off().hook_window(edit2.as_widget().as_win32());

        window.show();
        Ok(Self {
            window,
            edit,
            edit2,
        })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        // listen to events
        start! {
            sender, default: MainMessage::Noop,
            self.window => {
                WindowEvent::Close => MainMessage::Close,
            },
            self.edit => {
                EditEvent::Change => MainMessage::EditChange,
            },
            self.edit2 => {
                EditEvent::Change => MainMessage::Edit2Change,
            }
        }
    }

    async fn update_children(&mut self) -> Result<bool> {
        // update the window
        update_children!(self.window)
    }

    async fn update(
        &mut self,
        message: Self::Message,
        sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        // deal with custom messages
        match message {
            MainMessage::Noop => Ok(false),
            MainMessage::Close => {
                // the root component output stops the application
                sender.output(());
                // need not to call `render`
                Ok(false)
            }
            MainMessage::EditChange => {
                // handle edit change
                Ok(false)
            }
            MainMessage::Edit2Change => {
                // handle edit2 change
                Ok(false)
            }
        }
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) -> Result<()> {
        let csize = self.window.client_size()?;
        // adjust layout and draw widgets here
        let mut layout = layout! {
            Grid::from_str("auto", "auto,auto,1*").unwrap(),
            self.edit => { column: 0, row: 0 },
            self.edit2 => { column: 0, row: 1 },
        };
        layout.set_size(csize);
        Ok(())
    }

    fn render_children(&mut self) -> Result<()> {
        self.window.render();
        self.edit.render();
        self.edit2.render();
        Ok(())
    }
}
