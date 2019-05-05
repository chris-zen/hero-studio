use std::collections::{BTreeMap, HashMap};

use crate::{song::clips::ClipId, time::TicksTime};

type Key = u8;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Note {
  key: Key,
  velocity: f64,
  start: TicksTime,
  length: TicksTime,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum NoteEvent {
  NoteStart {
    key: Key,
    velocity: f64,
    end: TicksTime,
  },

  NoteEnd {
    key: Key,
    velocity: f64,
    start: TicksTime,
  },
}

type NoteEvents = Vec<NoteEvent>;

pub struct NotesClip {
  events: BTreeMap<TicksTime, NoteEvents>,
}

impl Default for NotesClip {
  fn default() -> Self {
    NotesClip {
      events: BTreeMap::new(),
    }
  }
}

impl NotesClip {
  pub fn new() -> NotesClip {
    NotesClip::default()
  }

  pub fn add_note(&mut self, note: Note) -> &mut Self {
    let (note_start_event, note_end_event, note_end) = self.split_note_into_events(&note);
    self.insert_note_event(note.start, note_start_event);
    self.insert_note_event(note_end, note_end_event);
    self
  }

  pub fn add_notes(&mut self, notes: Vec<Note>) -> &mut Self {
    notes.iter().for_each(|note| {
      self.add_note(*note);
    });
    self
  }

  pub fn remove_note(&mut self, note: Note) -> &mut Self {
    let (note_start_event, note_end_event, note_end) = self.split_note_into_events(&note);
    self.remove_note_event(note.start, &note_start_event);
    self.remove_note_event(note_end, &note_end_event);
    self
  }

  pub fn notes_range<'a>(
    &'a self,
    range_start: TicksTime,
    range_end: TicksTime,
  ) -> impl Iterator<Item = Note> + 'a {
    self
      .events
      .range(range_start..range_end)
      .flat_map(move |(tick, tick_events)| {
        tick_events.iter().flat_map(move |event| match *event {
          NoteEvent::NoteStart { key, velocity, end } => Some(Note {
            key,
            velocity,
            start: *tick,
            length: end - *tick,
          }),
          NoteEvent::NoteEnd {
            key,
            velocity,
            start,
          } => Some(Note {
            key,
            velocity,
            start,
            length: *tick - start,
          })
          .filter(|_event| start < range_start),
        })
      })
  }

  pub fn events_range<'a>(
    &'a self,
    start: TicksTime,
    end: TicksTime,
  ) -> impl Iterator<Item = &'a NoteEvent> + 'a {
    self
      .events
      .range(start..end)
      .flat_map(|(_tick, tick_events)| tick_events.iter().map(move |event| event))
  }

  fn split_note_into_events(&self, note: &Note) -> (NoteEvent, NoteEvent, TicksTime) {
    let note_end = note.start + note.length;

    let note_start_event = NoteEvent::NoteStart {
      key: note.key,
      velocity: note.velocity,
      end: note_end,
    };

    let note_end_event = NoteEvent::NoteEnd {
      key: note.key,
      velocity: note.velocity,
      start: note.start,
    };

    (note_start_event, note_end_event, note_end)
  }

  fn insert_note_event(&mut self, tick: TicksTime, event: NoteEvent) {
    self
      .events
      .entry(tick)
      .and_modify(|tick_events| tick_events.push(event))
      .or_insert_with(|| vec![event]);
  }

  fn remove_note_event(&mut self, tick: TicksTime, event: &NoteEvent) {
    if let Some(tick_events) = self.events.get_mut(&tick) {
      if let Some(index) = tick_events
        .iter()
        .position(|prev_event| prev_event == event)
      {
        tick_events.swap_remove(index);
        if tick_events.is_empty() {
          self.events.remove(&tick);
        }
      }
    }
  }
}

pub struct NotesSource {
  clips: HashMap<ClipId, NotesClip>,
}

#[cfg(test)]
mod test {

  use super::{BTreeMap, Note, NoteEvent, NoteEvents, NotesClip, TicksTime};

  #[test]
  /// NotesClip should add notes as events and allow repeated notes
  pub fn notes_clip_add_note() {
    let mut clip = NotesClip::new();
    let note1 = Note {
      key: 24,
      velocity: 1.0,
      start: TicksTime::new(1),
      length: TicksTime::new(2),
    };
    let note2 = Note {
      key: 25,
      velocity: 1.0,
      start: TicksTime::new(2),
      length: TicksTime::new(2),
    };
    let note3 = Note {
      key: 25,
      velocity: 1.0,
      start: TicksTime::new(2),
      length: TicksTime::new(2),
    };
    clip.add_note(note1);
    clip.add_note(note2);
    clip.add_note(note3);

    let mut expected_events: BTreeMap<TicksTime, NoteEvents> = BTreeMap::new();
    expected_events.insert(
      TicksTime::new(1),
      vec![NoteEvent::NoteStart {
        key: 24,
        velocity: 1.0,
        end: TicksTime::new(3),
      }],
    );
    expected_events.insert(
      TicksTime::new(2),
      vec![
        NoteEvent::NoteStart {
          key: 25,
          velocity: 1.0,
          end: TicksTime::new(4),
        },
        NoteEvent::NoteStart {
          key: 25,
          velocity: 1.0,
          end: TicksTime::new(4),
        },
      ],
    );
    expected_events.insert(
      TicksTime::new(3),
      vec![NoteEvent::NoteEnd {
        key: 24,
        velocity: 1.0,
        start: TicksTime::new(1),
      }],
    );
    expected_events.insert(
      TicksTime::new(4),
      vec![
        NoteEvent::NoteEnd {
          key: 25,
          velocity: 1.0,
          start: TicksTime::new(2),
        },
        NoteEvent::NoteEnd {
          key: 25,
          velocity: 1.0,
          start: TicksTime::new(2),
        },
      ],
    );

    assert_eq!(clip.events, expected_events)
  }

  #[test]
  /// NotesClip should remove events when removing notes
  pub fn notes_clip_remove_note() {
    let mut clip = NotesClip::new();
    let note1 = Note {
      key: 24,
      velocity: 1.0,
      start: TicksTime::new(1),
      length: TicksTime::new(2),
    };
    let note2 = Note {
      key: 25,
      velocity: 1.0,
      start: TicksTime::new(2),
      length: TicksTime::new(2),
    };
    let note3 = Note {
      key: 25,
      velocity: 1.0,
      start: TicksTime::new(2),
      length: TicksTime::new(2),
    };
    clip.add_notes(vec![note1, note2, note3]);

    clip.remove_note(note3);
    clip.remove_note(note1);

    let mut expected_events: BTreeMap<TicksTime, NoteEvents> = BTreeMap::new();
    expected_events.insert(
      TicksTime::new(2),
      vec![NoteEvent::NoteStart {
        key: 25,
        velocity: 1.0,
        end: TicksTime::new(4),
      }],
    );
    expected_events.insert(
      TicksTime::new(4),
      vec![NoteEvent::NoteEnd {
        key: 25,
        velocity: 1.0,
        start: TicksTime::new(2),
      }],
    );

    assert_eq!(clip.events, expected_events)
  }

  #[test]
  /// NotesClip should iterate notes over a range of ticks
  pub fn notes_clip_notes_range() {
    let mut clip = NotesClip::new();
    let note1 = Note {
      key: 21,
      velocity: 1.0,
      start: TicksTime::new(0),
      length: TicksTime::new(1),
    };
    let note2 = Note {
      key: 22,
      velocity: 1.0,
      start: TicksTime::new(1),
      length: TicksTime::new(2),
    };
    let note3 = Note {
      key: 23,
      velocity: 1.0,
      start: TicksTime::new(2),
      length: TicksTime::new(2),
    };
    let note4 = Note {
      key: 24,
      velocity: 1.0,
      start: TicksTime::new(1),
      length: TicksTime::new(4),
    };
    let note5 = Note {
      key: 24,
      velocity: 1.0,
      start: TicksTime::new(4),
      length: TicksTime::new(3),
    };
    clip.add_notes(vec![note1, note2, note3, note4, note5]);

    let range_result: Vec<Note> = clip
      .notes_range(TicksTime::new(2), TicksTime::new(5))
      .collect();
    assert_eq!(range_result, vec![note3, note2, note5])
  }

  #[test]
  /// NotesClip should add notes as events and allow repeated notes
  pub fn notes_clip_events_range() {
    let mut clip = NotesClip::new();
    let note1 = Note {
      key: 21,
      velocity: 1.0,
      start: TicksTime::new(0),
      length: TicksTime::new(1),
    };
    let note2 = Note {
      key: 22,
      velocity: 1.0,
      start: TicksTime::new(1),
      length: TicksTime::new(2),
    };
    let note3 = Note {
      key: 23,
      velocity: 1.0,
      start: TicksTime::new(2),
      length: TicksTime::new(2),
    };
    let note4 = Note {
      key: 24,
      velocity: 1.0,
      start: TicksTime::new(1),
      length: TicksTime::new(4),
    };
    let note5 = Note {
      key: 24,
      velocity: 1.0,
      start: TicksTime::new(4),
      length: TicksTime::new(3),
    };
    clip.add_notes(vec![note1, note2, note3, note4, note5]);

    let event1 = NoteEvent::NoteStart {
      key: 23,
      velocity: 1.0,
      end: TicksTime::new(4),
    };
    let event2 = NoteEvent::NoteEnd {
      key: 22,
      velocity: 1.0,
      start: TicksTime::new(1),
    };
    let event3 = NoteEvent::NoteEnd {
      key: 23,
      velocity: 1.0,
      start: TicksTime::new(2),
    };
    let event4 = NoteEvent::NoteStart {
      key: 24,
      velocity: 1.0,
      end: TicksTime::new(7),
    };
    let expected_events = vec![&event1, &event2, &event3, &event4];

    let range_result: Vec<&NoteEvent> = clip
      .events_range(TicksTime::new(2), TicksTime::new(5))
      .collect();
    assert_eq!(range_result, expected_events)
  }
}
