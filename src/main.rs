use futures::try_ready;
use tokio::prelude::*;
use tokio::timer::DelayQueue;

use std::time::{Duration, Instant};

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
    type Error = ();
    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        let mut r = try_ready!(self.q.poll().map_err(|_| ()))
            .unwrap()
            .into_inner();
        r.modify();

        self.q.insert_at(r.clone(), Instant::now() + r.duration);
        Ok(Async::Ready(Some(r.clone())))
    }
}

fn main() {
    let mut queue = DelayQueue::new();
    queue.insert_at(Item::new("1", Duration::from_secs(5)), Instant::now());
    queue.insert_at(Item::new("2", Duration::from_secs(25)), Instant::now());
    queue.insert_at(Item::new("3", Duration::from_secs(145)), Instant::now());

    let mq = MyQueue { q: queue };
    tokio::run(mq.for_each(|x| Ok(println!("{:?}", x))));
}
