use futures::ready;
use futures::{Stream, StreamExt};
use std::{pin::Pin, task::Poll};
use tokio::prelude::*;
use tokio::time::{DelayQueue, Instant};

use std::time::Duration;

#[derive(Debug, Clone)]
struct Item {
    what: String,
    duration: Duration,
    init: Duration,
}

impl Item {
    fn new(what: impl Into<String>, init: Duration) -> Self {
        Self {
            what: what.into(),
            duration: init,
            init,
        }
    }

    fn modify(&mut self) {
        self.duration = self
            .duration
            .checked_sub(Duration::from_secs(2))
            .unwrap_or(self.init);
    }
}

struct MyQueue {
    q: DelayQueue<Item>,
}

impl Stream for MyQueue {
    type Item = Item;
    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let mut r = ready!(Pin::new(&mut self.q).poll_next(cx))
            .unwrap()
            .unwrap()
            .into_inner();
        r.modify();

        self.q.insert_at(r.clone(), Instant::now() + r.duration);
        Poll::Ready(Some(r))
    }
}

#[tokio::main]
async fn main() {
    let mut queue = DelayQueue::new();
    queue.insert_at(Item::new("1", Duration::from_secs(5)), Instant::now());
    queue.insert_at(Item::new("2", Duration::from_secs(25)), Instant::now());
    queue.insert_at(Item::new("3", Duration::from_secs(145)), Instant::now());

    let mq = MyQueue { q: queue };
    mq.for_each(|x| async move { println!("{:?}", x) }).await;
}
