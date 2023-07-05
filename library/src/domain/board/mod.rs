pub mod commands;
pub mod entity;
pub mod events;
use std::{collections::VecDeque, mem};

use crate::aggregate;
use crate::utils::{ApplicationError, ApplicationResult};

use self::commands::{AddComment, CreateBoard, EditBoard, EditComment};
use self::entity::{Board, BoardState, Comment};
use self::events::{BoardCommentAdded, BoardCreated, BoardUpdated};

use super::builder::{Buildable, Builder};

use super::{Aggregate, Message};

#[derive(Default)]
pub struct BoardAggregate {
    pub board: Board,                   // Root
    pub comments: Vec<Comment>,         // Entity
    events: VecDeque<Box<dyn Message>>, //Event
}

impl BoardAggregate {
    pub fn create_board(&mut self, cmd: CreateBoard) {
        self.board = Board::new(cmd.author, cmd.title, cmd.content, cmd.state);
        self.raise_event(Box::new(BoardCreated {
            id: self.board.id,
            author: self.board.author,
            title: self.board.title.clone(),
            content: self.board.content.clone(),
            state: self.board.state.clone(),
        }))
    }
    pub fn update_board(&mut self, cmd: EditBoard) {
        if let Some(ref title) = cmd.title {
            self.board.title = title.clone();
        }
        if let Some(ref content) = cmd.content {
            self.board.content = content.clone();
        }
        if let Some(ref state) = cmd.state {
            self.board.state = state.clone()
        }
        self.raise_event(Box::new(BoardUpdated {
            id: self.board.id,
            title: cmd.title,
            content: cmd.content,
            state: cmd.state,
        }))
    }
    pub fn add_comment(&mut self, cmd: AddComment) {
        let new_comment = Comment::new(self.board.id, cmd.author, &cmd.content);
        self.raise_event(Box::new(BoardCommentAdded {
            id: new_comment.id,
            author: new_comment.author,
            content: new_comment.content.clone(),
            state: new_comment.state.clone(),
        }));
        self.comments.push(new_comment);
    }

    pub fn delete(&mut self) {
        self.board.state = BoardState::Deleted
    }

    pub fn edit_comment(&mut self, cmd: EditComment) -> ApplicationResult<()> {
        let comment = self
            .comments
            .iter_mut()
            .find(|c| c.id == cmd.id)
            .ok_or(ApplicationError::EntityNotFound)?;
        comment.content = cmd.content;
        Ok(())
    }
}

aggregate!(BoardAggregate);

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
