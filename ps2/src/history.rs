use std::collections::VecDeque;
use std::cmp::min;

pub struct History {
    capacity: usize,
    history: VecDeque<String>
}

impl History {
    pub fn new(capacity: usize) -> Self {
        History {
            capacity: capacity,
            history: VecDeque::with_capacity(capacity)
        }
    }

    pub fn add(&mut self, cmd: String) {
        self.history.push_front(cmd);
        if self.history.len() > self.capacity {
            self.history.pop_back();
        }
    }

    pub fn list(&self, n: usize) -> &[String] {
        let (front, _) = self.history.as_slices();
        let idx = min(n, front.len());
        let (limit, _) = front.split_at(idx);
        limit
    }
}

#[cfg(test)]
mod test {
    use super::History;

    #[test]
    fn list_returns_most_recent_n_items_and_does_not_pop() {
        let mut history = History::new(4);
        history.add("Hello world".to_string());
        history.add("Goodbye world".to_string());
        history.add("Hello moon".to_string());

        assert_eq!(history.list(2), vec!["Hello moon".to_string(), "Goodbye world".to_string()].as_slice())
    }

    #[test]
    fn does_not_grow_beyond_provided_capacity() {
        let mut history = History::new(4);
        history.add("Hello world".to_string());
        history.add("Goodbye world".to_string());
        history.add("Hello moon".to_string());
        history.add("Goodbye moon".to_string());
        history.add("Hello mars".to_string());

        assert_eq!(
            history.list(5),
            vec![
                "Hello mars".to_string(),
                "Goodbye moon".to_string(),
                "Hello moon".to_string(),
                "Goodbye world".to_string(),
            ].as_slice()
        );
    }
}
