#[derive(Debug, PartialEq)]
pub enum Status {
    Todo,
    Done,
}

impl Status {
    pub fn toggle(&self) -> Self {
        match self {
            Status::Todo => Status::Done,
            Status::Done => Status::Todo,
        }
    }
}

pub fn parse_item(line: &str) -> Option<(Status, &str)> {
    let todo_item = line
        .strip_prefix("TODO: ")
        .map(|title| (Status::Todo, title));
    let done_item = line
        .strip_prefix("DONE: ")
        .map(|title| (Status::Done, title));
    todo_item.or(done_item)
}

pub fn list_drag_up(list: &mut [String], list_curr: &mut usize) {
    if *list_curr > 0 {
        list.swap(*list_curr, *list_curr - 1);
        *list_curr -= 1;
    }
}

pub fn list_drag_down(list: &mut [String], list_curr: &mut usize) {
    if *list_curr + 1 < list.len() {
        list.swap(*list_curr, *list_curr + 1);
        *list_curr += 1;
    }
}

pub fn list_up(list_curr: &mut usize) {
    if *list_curr > 0 {
        *list_curr -= 1;
    }
}

pub fn list_down(list: &[String], list_curr: &mut usize) {
    if *list_curr + 1 < list.len() {
        *list_curr += 1;
    }
}

pub fn list_first(list_curr: &mut usize) {
    if *list_curr > 0 {
        *list_curr = 0;
    }
}

pub fn list_last(list: &[String], list_curr: &mut usize) {
    if !list.is_empty() {
        *list_curr = list.len() - 1;
    }
}

pub fn list_transfer(
    list_dst: &mut Vec<String>,
    list_src: &mut Vec<String>,
    list_src_curr: &mut usize,
) {
    if *list_src_curr < list_src.len() {
        list_dst.push(list_src.remove(*list_src_curr));
        if *list_src_curr >= list_src.len() && !list_src.is_empty() {
            *list_src_curr = list_src.len() - 1;
        }
    }
}
