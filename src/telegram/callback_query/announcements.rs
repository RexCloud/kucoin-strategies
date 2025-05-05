use teloxide::{
    payloads::AnswerCallbackQuerySetters as _, prelude::Requester as _, types::CallbackQuery, Bot,
    RequestError,
};

use crate::kucoin::{announcements::AnnouncementType, KuCoin};

pub async fn toggle(
    bot: Bot,
    query: CallbackQuery,
    kucoin: KuCoin,
    r#type: AnnouncementType,
) -> Result<(), RequestError> {
    let notifiable: Vec<AnnouncementType> = {
        let mut notifiable = kucoin.announcements().notifiable();

        if !notifiable.remove(&r#type) {
            notifiable.insert(r#type);
        }

        notifiable.iter().copied().collect()
    };

    let text = match notifiable.is_empty() {
        true => "Notifications are disabled",
        false => &serde_json::to_string_pretty(&notifiable).unwrap(),
    };

    bot.answer_callback_query(query.id).text(text).await?;

    Ok(())
}
