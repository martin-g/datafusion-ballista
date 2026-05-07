// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

use crate::tui::domain::{
    SchedulerState,
    executors::Executor,
    jobs::{
        Job, JobDetails,
        stages::{JobStagesResponse, StagesGraph},
    },
    metrics::Metric,
};
#[cfg(feature = "tui")]
use crossterm::event::{Event, EventStream, KeyEvent};
use ratatui::Terminal;
#[cfg(feature = "tui-web")]
use ratzilla::event::KeyEvent;
use ratzilla::{DomBackend, WebRenderer};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc;

#[derive(Clone, Debug)]
pub enum UiData {
    Executors(Option<SchedulerState>, Vec<Executor>, Vec<Job>),
    Metrics(Vec<Metric>),
    Jobs(Vec<Job>),
    JobDetails(JobDetails),
    JobStagesGraph(StagesGraph),
    JobStagesData(String, JobStagesResponse),
}

#[derive(Clone, Debug)]
pub enum Event {
    Key(KeyEvent),
    Tick,
    DataLoaded { data: UiData },
}

#[derive(Debug)]
pub struct EventHandler {
    rx: mpsc::UnboundedReceiver<Event>,
}

impl EventHandler {
    #[cfg(feature = "tui")]
    pub fn new(tick_rate: Duration) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            #[cfg(feature = "tui")]
            let mut reader = EventStream::new();
            let mut interval = tokio::time::interval(tick_rate);

            loop {
                let interval_delay = interval.tick();
                let crossterm_event = reader.next().fuse();

                #[cfg(feature = "tui")]
                tokio::select! {
                    _ = interval_delay => {
                        if tx.send(Event::Tick).is_err() {
                            break;
                        }
                    }
                    Some(Ok(evt)) = crossterm_event => {
                      if let crossterm::event::Event::Key(key) = evt
                          && key.kind == crossterm::event::KeyEventKind::Press
                              && tx.send(Event::Key(key)).is_err()
                          {
                              break;
                          }
                    }
                }
            }
        });

        Self { rx }
    }

    #[cfg(feature = "tui-web")]
    pub fn new(tick_rate: Duration, terminal: &Terminal<DomBackend>) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        struct RatzillaFuture {
            key_events: Arc<Mutex<VecDeque<KeyEvent>>>,
        }

        impl Future for RatzillaFuture {
            type Output = KeyEvent;

            fn poll(
                self: std::pin::Pin<&mut Self>,
                cx: &mut std::task::Context<'_>,
            ) -> std::task::Poll<Self::Output> {
                if let Some(key_event) = self.key_events.lock().unwrap().pop_front() {
                    return std::task::Poll::Ready(key_event);
                }
                cx.waker().wake_by_ref();
                std::task::Poll::Pending
            }
        }

        unsafe impl Send for RatzillaFuture {}
        unsafe impl Sync for RatzillaFuture {}

        let key_events = Arc::new(Mutex::new(VecDeque::new()));
        let terminal_key_events = key_events.clone();
        terminal.on_key_event(move |key_event| {
            terminal_key_events.lock().unwrap().push_back(key_event)
        });

        let future_key_events = key_events.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tick_rate);

            loop {
                let interval_delay = interval.tick();
                let ratzilla_future = RatzillaFuture {
                    key_events: future_key_events.clone(),
                };

                tokio::select! {
                    _ = interval_delay => {
                        if tx.send(Event::Tick).is_err() {
                            break;
                        }
                    }
                    key = ratzilla_future => {
                      if tx.send(Event::Key(key)).is_err()
                          {
                              break;
                          }
                    }
                }
            }
        });

        Self { rx }
    }

    pub async fn next(&mut self) -> Option<Event> {
        self.rx.recv().await
    }
}
