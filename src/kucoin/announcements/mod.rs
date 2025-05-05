use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    fmt,
    sync::{Arc, Mutex, MutexGuard},
    time::{Duration, UNIX_EPOCH},
};
use strum::{EnumString, VariantNames};
use teloxide::{
    payloads::SendMessageSetters as _, prelude::Requester as _, types::ParseMode::Html, Bot,
};
use tracing::error;

use crate::kucoin::{constants::ANNOUNCEMENTS, response::Paginated, task::Poller, Request};

#[derive(Debug, Clone)]
pub struct Announcements {
    notifiable: Arc<Mutex<BTreeSet<AnnouncementType>>>,
    period: Duration,
}

impl Default for Announcements {
    fn default() -> Self {
        Announcements {
            notifiable: Default::default(),
            period: Duration::from_secs(100),
        }
    }
}

impl Announcements {
    pub fn notifiable(&self) -> MutexGuard<'_, BTreeSet<AnnouncementType>> {
        self.notifiable.lock().unwrap()
    }

    pub fn period(&self) -> Duration {
        self.period
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    EnumString,
    VariantNames,
    Serialize,
    Deserialize,
)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "title_case")]
pub enum AnnouncementType {
    LatestAnnouncements,
    FuturesAnnouncements,
    Activities,
    NewListings,
    ProductUpdates,
    Vip,
    MaintenanceUpdates,
    ApiCampaigns,
    Delistings,
    Others,
}

impl Poller for (Announcements, Bot) {
    async fn poll(&self, client: &Client) {
        #[derive(Debug, Serialize, Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Announcement {
            ann_id: u32,
            ann_title: String,
            ann_type: Vec<AnnouncementType>,
            ann_desc: String,
            ann_url: String,
            c_time: u64,
            language: String,
        }

        impl fmt::Display for Announcement {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(
                    f,
                    r#"📣 {} - <a href="{}">link</a>"#,
                    self.ann_title, self.ann_url
                )
            }
        }

        let (announcements, bot) = (&self.0, &self.1);

        let path = format!(
            "{ANNOUNCEMENTS}?startTime={}",
            (UNIX_EPOCH.elapsed().unwrap() - announcements.period()).as_millis()
        );

        match Request::get(path)
            .send::<Paginated<Announcement>>(client)
            .await
        {
            Ok(paginated) => {
                let announcements: Vec<Announcement> = {
                    let notifiable = announcements.notifiable();

                    paginated
                        .into_iter()
                        .filter(|announcement| {
                            announcement
                                .ann_type
                                .iter()
                                .any(|r#type| notifiable.contains(r#type))
                        })
                        .collect()
                };

                for announcement in announcements {
                    if let Err(e) = bot
                        .send_message(env!("USER_ID").to_string(), announcement.to_string())
                        .parse_mode(Html)
                        .await
                    {
                        error!("{e}")
                    }
                }
            }
            Err(e) => error!("{e}"),
        }
    }
}
