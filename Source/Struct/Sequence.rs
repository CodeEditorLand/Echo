pub struct Struct {
	Site: Arc<dyn Worker>,
	Work: Arc<Production::Struct>,
	Life: Life::Struct,
	Time: Signal::Struct<bool>,
}

impl Struct {
	pub fn New(Site: Arc<dyn Worker>, Work: Arc<Production>, Context: Life::Struct) -> Self {
		Struct { Site, Work, Life: Context, Time: Signal::Struct::New(false) }
	}

	pub async fn Run(&self) {
		while !self.Time.Get().await {
			if let Some(Action) = self.Work.Do().await {
				let Result = self.Again(Action).await;

				if let Err(e) = Result {
					error!("Error processing action: {}", e);
				}
			}
		}
	}

	async fn Again(&self, Action: Box<dyn ActionTrait>) -> Result<(), ActionError> {
		let MaxRetries = self.Life.Fate.get_int("max_retries").unwrap_or(3) as u32;

		let mut Retries = 0;

		loop {
			match self.Site.Receive(Action.Clone(), &self.Life).await {
				Ok(_) => return Ok(()),
				Err(e) => {
					if Retries >= MaxRetries {
						return Err(e);
					}
					Retries += 1;

					let Delay = Duration::from_secs(
						2u64.pow(Retries) + rand::thread_rng().gen_range(0..1000),
					);

					warn!(
						"Action failed, retrying in {:?}. Attempt {} of {}",
						Delay, Retries, MaxRetries
					);

					sleep(Delay).await;
				}
			}
		}
	}

	pub async fn Shutdown(&self) {
		self.Time.Set(true).await;
	}
}

use config::{Config, File};
use log::{error, warn};
use metrics::{counter, gauge};
use rand::Rng;
use std::{sync::Arc, time::Duration};
use thiserror::Error;
use tokio::{
	sync::Mutex,
	time::{sleep, Duration},
};

pub mod Action;
pub mod Life;
pub mod Plan;
pub mod Production;
pub mod Signal;
pub mod Vector;
