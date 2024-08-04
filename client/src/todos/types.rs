#[derive(Debug, Default)]
pub struct TodoItemTime {
	duration: u16,
	hour: u16,
	minutes: u16,
}

#[derive(Debug, Default)]
pub struct TodoItem {
	message: String,
	time: Option<TodoItemTime>,
}

#[derive(Debug, Default)]
pub struct TodoDay {
	items: Vec<TodoItem>,
}

#[derive(Debug, Default)]
pub struct TodosState {
	days: Vec<TodoDay>,
	general: TodoDay,
}
