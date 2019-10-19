use diesel::{Connection, SqliteConnection};
use kafi_kaesseli_core::currency_handling::currency_formatter::CurrencyFormatterImpl;
use kafi_kaesseli_core::currency_handling::currency_parser::CurrencyParserImpl;
use kafi_kaesseli_core::message_handler::MessageHandlerImpl;
use kafi_kaesseli_core::message_router::MessageRouterImpl;
use kafi_kaesseli_core::services::product_service::ProductServiceImpl;
use kafi_kaesseli_core::services::user_service::UserServiceImpl;
use tbot::types::parameters::Text;
use tbot::types::{message, update, Message};
use tbot::{prelude::*, Bot};

fn main() {
    let database_connection = SqliteConnection::establish("database.sqlite").unwrap();

    let message_handler = MessageHandlerImpl::new(
        &database_connection,
        Box::new(MessageRouterImpl::new(
            Box::new(ProductServiceImpl::new(&database_connection)),
            Box::new(CurrencyParserImpl::new()),
        )),
        Box::new(CurrencyFormatterImpl::new()),
        Box::new(UserServiceImpl::new(&database_connection)),
    );

    let mut bot = Bot::from_env("BOT_TOKEN").event_loop();

    bot.unhandled(|context| match &context.update {
        update::Kind::Message(Message {
            chat,
            kind: message::Kind::Text(text),
            ..
        }) => {
            let reply = context
                .bot
                .send_message(chat.id, Text::plain(&text.value))
                .into_future()
                .map_err(|err| {
                    dbg!(err);
                });

            tbot::spawn(reply);
        }
        _ => {}
    });

    bot.polling().start();
}
