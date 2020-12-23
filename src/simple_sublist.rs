use std::sync::{Arc};
use tokio::sync::Mutex;
use crate::error::*;
use std::collections::{HashMap, BTreeSet};
use bitflags::_core::cmp::Ordering;
use crate::parse::ParseResult::Sub;
use bitflags::_core::option::Option::Some;
use crate::client::ClientMessageSender;

#[derive(Debug)]
pub struct Subscription {
    pub msg_sender: Arc<Mutex<ClientMessageSender>>,
    pub subject: String,
    pub queue: Option<String>,
    pub sid: String,
}

#[derive(Debug, Default)]
pub struct SubResult {
    pub subs: Vec<ArcSubscription>,
    pub qsubs: Vec<ArcSubscription>,
}

pub type ArcSubscription = Acr<Subscription>;
pub type ArcSubResult = Arc<SubResult>;

pub trait SubListTrait {
    fn insert(&mut self, sub: ArcSubscription) -> Result<()>;
    fn remove(&mut self, sub: ArcSubscription) -> Result<()>;
    fn match_subject(&mut self, subject: &str) -> Result<ArcSubResult>;
}

#[derive(Debug)]
pub struct SimpleSubList {
    subs: HashMap<String, BTreeSet<ArcSubscriptionWrapper>>,
    qsubs: HashMap<String, HashMap<String, BTreeSet<ArcSubscriptionWrapper>>>,
}

#[derive(Clone, Debug)]
pub struct ArcSubscriptionWrapper(ArcSubscription);

impl std::cmp::Ord for ArcSubscriptionWrapper {
    fn cmp(&self, other: &Self) -> Ordering {
        let a = self.0.as_ref() as *const Subscription as usize;
        let b = other.0.as_ref()
            as *const Subscription as usize;
        a.cmp(&b)
    }
}

impl std::cmp::PartialOrd for ArcSubscriptionWrapper {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::PartialEq for ArcSubscriptionWrapper {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl std::cmp::Eq for ArcSubscriptionWrapper {}

impl SubListTrait for SimpleSubList {
    fn insert(&mut self, sub: Arc<Subscription>) -> Result<()> {
        if let Some(ref q) = sub.queue {
            let entry = self
                .qsubs
                .entry(sub.subject.clone())
                .or_insert(Default::default());
            let queue = entry.entry(q.clone()).or_insert(Default::default());
            queue.insert(ArcSubscriptionWrapper(sub));
        } else {
            let subs = self
                .subs
                .entry(sub.subject.clone())
                .or_insert(Default::default());
            subs.insert(ArcSubscriptionWrapper(sub))
        }
        Ok(())
    }

    fn remove(&mut self, sub: Arc<Subscription>) -> Result<()> {
        if let Some(ref q) = sub.queue {
            if let Some(subs) = self.qsubs.get_mut(&sub.subject) {
                if let Some(qsubs) = subs.get_mut(q) {
                    qsubs.remove(&ArcSubscriptionWrapper(sub.clone()));
                    if qsubs.is_empty() {
                        subs.remove(q);
                    }
                } else {
                    return Err(RmqError::new(ERROR_SUBSCRIPTION_NOT_FOUND));
                }
                if subs.is_empty() {
                    self.qsubs.remove(&sub.subject);
                }
            } else {
                return Err(RmqError::new(ERROR_SUBSCRIPTION_NOT_FOUND));
            }
        } else {
            if let Some(subs) = self.subs.get_mut(&sub.subject) {
                subs.remove(&ArcSubscriptionWrapper(sub.clone()));
                if subs.is_empty() {
                    self.subs.remove(&sub.subject);
                }
            }
        }

        Ok(())
    }

    fn match_subject(&mut self, subject: &str) -> Result<ArcSubResult> {
        let mut r = SubResult::default();
        if let Some(subs) = self.subs.get(subject) {
            for s in subs {
                r.subs.push(s.0.clone());
            }
        }
        if let Some(qsub) = self.qsubs.get(subject) {
            for (_, qsun) in qsub {
                let mut v = Vec::with_capacity(qsub.len());
                for s in qsub {
                    v.push(s.0.clone());
                }
                r.qsubs.push(v);
            }
        }
        Ok(Arc::new(r));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::new_test_tcp_writer;

    #[tokio::main]
    #[test]
    async fn test_match() {
        let mut sl = SimpleSubList::default();
        let mut subs = Vec::new();
        let r = sl.match_subject("test").unwrap();
        assert_eq!(r.subs.len(), 0);
        assert_eq!(r.qsubs.len(), 0);
        let sub = Arc::new(Subscription::new("test", None, "1", new_test_tcp_writer()));
        subs.push(sub.clone());
        let r = sl.insert(sub);
        assert!(!r.is_err());
        let r = sl.match_subject("test").unwrap();
        assert_eq!(r.subs.len(), 1);
        assert_eq!(r.qsubs.len(), 0);
        let sub = Arc::new(Subscription::new("test", None, "1", new_test_tcp_writer()));
        subs.push(sub.clone());
        let r = sl.insert(sub);
        assert!(!r.is_err());
        let r = sl.match_subject("test").unwrap();
        assert_eq!(r.subs.len(), 2);
        assert_eq!(r.qsubs.len(), 0);
        let sub = Arc::new(Subscription::new(
            "test",
            Some("q"),
            "1",
            new_test_tcp_writer(),
        ));
        subs.push(sub.clone());
        let r = sl.insert(sub);
        assert!(!r.is_err());
        let r = sl.match_subject("test").unwrap();
        assert_eq!(r.subs.len(), 2);
        assert_eq!(r.qsubs.len(), 1);
        let sub = Arc::new(Subscription::new(
            "test",
            Some("q"),
            "1",
            new_test_tcp_writer(),
        ));
        subs.push(sub.clone());
        let r = sl.insert(sub);
        assert!(!r.is_err());
        let r = sl.match_subject("test").unwrap();
        assert_eq!(r.subs.len(), 2);
        assert_eq!(r.qsubs.len(), 1);

        let sub = Arc::new(Subscription::new(
            "test",
            Some("q2"),
            "1",
            new_test_tcp_writer(),
        ));
        subs.push(sub.clone());
        let r = sl.insert(sub);
        assert!(!r.is_err());
        let r = sl.match_subject("test").unwrap();
        assert_eq!(r.subs.len(), 2);
        assert_eq!(r.qsubs.len(), 2);

        let s = subs.pop().unwrap();
        let r = sl.remove(s);
        assert!(!r.is_err());
        let r = sl.match_subject("test").unwrap();
        assert_eq!(r.subs.len(), 2);
        assert_eq!(r.qsubs.len(), 1);

        let s = subs.pop().unwrap();
        let r = sl.remove(s);
        assert!(!r.is_err());
        let r = sl.match_subject("test").unwrap();
        assert_eq!(r.subs.len(), 2);
        assert_eq!(r.qsubs.len(), 1);

        let s = subs.pop().unwrap();
        let r = sl.remove(s);
        assert!(!r.is_err());
        let r = sl.match_subject("test").unwrap();
        assert_eq!(r.subs.len(), 2);
        assert_eq!(r.qsubs.len(), 0);

        let s = subs.pop().unwrap();
        let r = sl.remove(s);
        assert!(!r.is_err());
        let r = sl.match_subject("test").unwrap();
        assert_eq!(r.subs.len(), 1);
        assert_eq!(r.qsubs.len(), 0);

        let s = subs.pop().unwrap();
        let r = sl.remove(s);
        assert!(!r.is_err());
        let r = sl.match_subject("test").unwrap();
        assert_eq!(r.subs.len(), 0);
        assert_eq!(r.qsubs.len(), 0);
    }
}
