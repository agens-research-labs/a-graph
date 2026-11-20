//! `blackboards`: shared state between agents. The content is always Markdown.

use super::key::*;
use super::{Field, Kind, Shape};

const BLACKBOARD_ITEM: Shape = Shape::Map(&[
    Field::req(ID, Shape::Id(Kind::Blackboard)).doc("The id of the blackboard, cited by the nodes and used as the placeholder of its text."),
    Field::req(
        SOURCE,
        Shape::Either(&[
            Shape::Words(&["memory"]),
            Shape::Map(&[Field::req(FILE, Shape::Text).doc(
                "A Markdown file, which must exist when the file is checked and start with its header: a `<description>` and a `<purpose>`, each followed by a text, between two `---` lines (`blackboard_header`).",
            )]),
            Shape::Map(&[
                Field::req(URL, Shape::Text).doc("A URL. It is not supported: the file is refused when it is checked (`not_supported`), since a board is written as well as read."),
                Field::opt(ACCESS, Shape::Any).doc("How to get in. Its content is not defined yet."),
            ]),
        ]),
    )
    .doc("Where the blackboard is kept: `memory`, in the program, empty at the start and lost at the end of the run; or a file. A URL is refused."),
]);

pub(super) const BLACKBOARDS: Shape = Shape::List(&BLACKBOARD_ITEM);
