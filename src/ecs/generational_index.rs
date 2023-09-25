use super::entities::EntityId;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct GenerationalIndex {
    index: EntityId,
    generation: usize,
}

impl GenerationalIndex {
    pub fn index(&self) -> usize {
        self.index
    }

    pub fn generation(&self) -> usize {
        self.generation
    }
}

struct AllocatorEntry {
    is_live: bool,
    generation: usize,
}

#[derive(Default)]
pub struct GenerationalIndexAllocator {
    entries: Vec<AllocatorEntry>,
    free: Vec<usize>,
}

impl GenerationalIndexAllocator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn allocate(&mut self) -> GenerationalIndex {
        if let Some(index) = self.free.pop() {
            let id_entry = &mut self.entries[index];
            assert!(!id_entry.is_live);
            id_entry.is_live = true;
            GenerationalIndex {
                index,
                generation: id_entry.generation,
            }
        } else {
            self.entries.push(AllocatorEntry {
                is_live: true,
                generation: 0,
            });
            GenerationalIndex {
                index: self.entries.len() - 1,
                generation: 0,
            }
        }
    }

    pub fn deallocate(&mut self, gen_index: GenerationalIndex) -> bool {
        if gen_index.index >= self.entries.len() {
            return false;
        }

        let id_entry = &mut self.entries[gen_index.index];
        if !id_entry.is_live {
            return false;
        }

        id_entry.is_live = false;
        id_entry.generation = id_entry
            .generation
            .checked_add(1)
            .expect("GenerationalIndex overflow");
        self.free.push(gen_index.index);
        true
    }

    pub fn live_count(&self) -> usize {
        self.entries.iter().filter(|gi| gi.is_live).count()
    }

    pub fn is_live(&self, gen_index: GenerationalIndex) -> bool {
        if gen_index.index >= self.entries.len() {
            return false;
        }
        self.entries[gen_index.index].is_live
            && self.entries[gen_index.index].generation == gen_index.generation
    }

    pub fn max_allocated(&self) -> usize {
        self.entries.len()
    }

    pub fn live_at_index(&self, index: usize) -> Option<GenerationalIndex> {
        self.entries.get(index).and_then(|gi| {
            if gi.is_live {
                return Some(GenerationalIndex {
                    index,
                    generation: gi.generation,
                });
            }
            None
        })
    }
}
