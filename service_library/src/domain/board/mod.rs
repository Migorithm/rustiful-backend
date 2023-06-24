pub mod entity;
pub mod events;

use std::{collections::VecDeque, mem};

use crate::utils::{ApplicationError, ApplicationResult};

use self::entity::{Board, BoardState, Comment};
use self::events::BoardEvent;
use super::builder::{Buildable, Builder};
use super::commands::ApplicationCommand;

use super::Aggregate;
use uuid::Uuid;

impl Aggregate for BoardAggregate {
    type Event = BoardEvent;

    fn events(&self) -> &VecDeque<Self::Event> {
        &self.events
    }
    fn take_events(&mut self) -> VecDeque<Self::Event> {
        mem::take(&mut self.events)
    }
    fn raise_event(&mut self, event: Self::Event) {
        self.events.push_back(event)
    }
}

#[derive(Default)]
pub struct BoardAggregate {
    pub board: Board,                 // Root
    pub comments: Vec<Comment>,       // Entity
    pub events: VecDeque<BoardEvent>, //Event
}

impl BoardAggregate {
    pub fn create_board(
        &mut self,
        author: Uuid,
        title: String,
        content: String,
        state: BoardState,
    ) {
        self.board = Board::new(author, title, content, state);
        self.raise_event(BoardEvent::Created {
            id: self.board.id,
            author: self.board.author,
            title: self.board.title.clone(),
            content: self.board.content.clone(),
            state: self.board.state.clone(),
        })
    }
    pub fn update_board(
        &mut self,
        title: Option<String>,
        content: Option<String>,
        state: Option<BoardState>,
    ) {
        if let Some(ref title) = title {
            self.board.title = title.clone();
        }
        if let Some(ref content) = content {
            self.board.content = content.clone();
        }
        if let Some(ref state) = state {
            self.board.state = state.clone()
        }
        self.raise_event(BoardEvent::Updated {
            id: self.board.id,
            title,
            content,
            state,
        })
    }
    pub fn add_comment(&mut self, author: Uuid, content: String) {
        self.comments
            .push(Comment::new(self.board.id, author, &content))
    }
    pub fn delete(&mut self) {
        self.board.state = BoardState::Deleted
    }

    fn edit_comment(&mut self, id: Uuid, content: String) -> ApplicationResult<()> {
        let comment = self
            .comments
            .iter_mut()
            .find(|c| c.id == id)
            .ok_or(ApplicationError::NotFound)?;
        comment.content = content;
        Ok(())
    }

    pub fn execute(&mut self, cmd: ApplicationCommand) -> ApplicationResult<()> {
        //TODO Event generation!
        match cmd {
            ApplicationCommand::CreateBoard {
                author,
                title,
                content,
                state,
            } => self.create_board(author, title, content, state),
            ApplicationCommand::EditBoard {
                title,
                content,
                state,
                ..
            } => self.update_board(title, content, state),

            ApplicationCommand::AddComment {
                author, content, ..
            } => self.add_comment(author, content),
            ApplicationCommand::EditComment { id, content, .. } => {
                self.edit_comment(id, content)?
            }
        }
        Ok(())
    }
}

impl AsRef<BoardAggregate> for BoardAggregate {
    fn as_ref(&self) -> &BoardAggregate {
        self
    }
}
impl AsMut<BoardAggregate> for BoardAggregate {
    fn as_mut(&mut self) -> &mut BoardAggregate {
        self
    }
}

pub struct BoardAggregateBuilder(BoardAggregate);

impl BoardAggregateBuilder {
    pub fn take_board(mut self, board: Board) -> Self {
        self.0.board = board;
        self
    }
    pub fn take_comments(mut self, comments: Vec<Comment>) -> Self {
        self.0.comments = comments;
        self
    }
}

impl Builder<BoardAggregate> for BoardAggregateBuilder {
    fn new() -> Self {
        Self(BoardAggregate::default())
    }

    fn build(self) -> BoardAggregate {
        self.0
    }
}

impl Buildable<BoardAggregate, BoardAggregateBuilder> for BoardAggregate {
    fn builder() -> BoardAggregateBuilder {
        BoardAggregateBuilder::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::board::entity::{Board, BoardState};

    use uuid::Uuid;

    #[test]
    fn test_create_board() {
        let board = Board::new(
            Uuid::new_v4(),
            "러스트를 배워야하는 이유",
            "졸잼임",
            BoardState::Published,
        );
        assert_eq!(board.state(), "Published");

        let board = Board::new(
            Uuid::new_v4(),
            "러스트를 배워야하는 이유",
            "졸잼임",
            BoardState::Unpublished,
        );
        assert_eq!(board.state(), "Unpublished");

        let board = Board::new(
            Uuid::new_v4(),
            "러스트를 배워야하는 이유",
            "졸잼임",
            BoardState::Deleted,
        );
        assert_eq!(board.state(), "Deleted");
    }
}
