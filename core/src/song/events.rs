
enum Event {
  NoteStart {
    key: u8,
    velocity: f64,
    end: TicksTime
  },

  NoteEnd {
    key: u8,
    velocity: f64,
    start: TicksTime
  }
}

pub struct NotesClip {
  // TODO BTreeMap<TicksTime, Set<Event>>
  events: BTreeMap<TicksTime, Event>
}

impl NotesClip {
  pub fn new() -> NotesClip {
    NotesClip {
      events: BTreeMap::new()
    }
  }

  pub fn add_note(&mut self, note: Note) -> &mut Self {
    let note_end = note.start + note.length;

    let note_start_event = Event::NoteStart {
      key: note.key,
      velocity: note.velocity,
      end: note_end
    };
    self.events.insert(note.start, note_start_event);

    let note_end_event = Event::NoteEnd {
      key: note.key,
      velocity: note.velocity,
      start: note.start
    };
    self.events.insert(note_end, note_end_event);

    self
  }

  // delete_note
}
