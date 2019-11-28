use crate::lang::{
    ast::{DietInstr, MachineInstr},
    Compiler,
};
use std::collections::HashMap;

impl Compiler<Vec<DietInstr>> {
    /// Strips the labels out of an instruction list, replacing them with the
    /// appropriate line numbers. E.g.:
    ///
    /// 0: Set(0)
    /// 1: Jez("label_0")
    /// 2: Set(1)
    /// 3: Label("label_0")
    /// 4: Set(2)
    ///
    /// maps to
    ///
    /// 0: Set(0)
    /// 1: Jez(3)
    /// 2: Set(1)
    /// 3: Set(2)
    pub fn delabel(self) -> Compiler<Vec<MachineInstr>> {
        // Build a mapping of label:index
        let label_map: HashMap<String, usize> = self
            .0
            .iter()
            // create a map of label:index
            .fold((0, HashMap::new()), |(idx, mut map), instr| {
                if let DietInstr::Label(label) = instr {
                    // Map the label to the index where it occurred. We need a
                    // clone here to get around the borrow checker, because it
                    // doesn't know this is the last time we'll need the label.
                    if let Some(dupe) = map.insert(label.clone(), idx) {
                        panic!(format!("Duplicate label {}", dupe));
                    }

                    // return idx instead of idx+1 because this Label
                    // instruction is going to be removed from the list
                    (idx, map)
                } else {
                    (idx + 1, map) // Just count this instruction and move on
                }
            })
            .1;
        let get_idx_for_label = |label: &String| -> usize {
            *label_map
                .get(label)
                .unwrap_or_else(|| panic!(format!("Unknown label: {}", label)))
        };

        // Use the label map to convert each instruction. For most instruction
        // types this is a 1:1 mapping, but for jumps, their labels need to be
        // replaced with the corresponding indexes.
        let machine_instrs: Vec<MachineInstr> = self
            .0
            .into_iter()
            .filter_map(|instr| match instr {
                // Basic instructions, just map 1:1
                DietInstr::Read => Some(MachineInstr::Read),
                DietInstr::Write => Some(MachineInstr::Write),
                DietInstr::Set(value) => Some(MachineInstr::Set(value)),
                DietInstr::Push(stack_id) => Some(MachineInstr::Push(stack_id)),
                DietInstr::Pop(stack_id) => Some(MachineInstr::Pop(stack_id)),

                // We don't want labels any more, throw them away
                DietInstr::Label(_) => None,

                // Jumps - replace labels with their corresponding indexes
                DietInstr::Jez(label) => {
                    Some(MachineInstr::Jez(get_idx_for_label(&label)))
                }
                DietInstr::Jnz(label) => {
                    Some(MachineInstr::Jnz(get_idx_for_label(&label)))
                }
            })
            .collect();

        Compiler(machine_instrs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        let compiler = Compiler(vec![
            DietInstr::Set(0),
            DietInstr::Jez("label_0".into()),
            DietInstr::Set(2),
            DietInstr::Label("label_0".into()),
            DietInstr::Set(1),
        ]);

        assert_eq!(
            compiler.delabel().0,
            vec![
                MachineInstr::Set(0),
                MachineInstr::Jez(3), // --+
                MachineInstr::Set(2), //   |
                MachineInstr::Set(1), // <-+
            ]
        );
    }

    #[test]
    fn test_multiple_labels() {
        let compiler = Compiler(vec![
            DietInstr::Set(0),
            DietInstr::Jez("label_0".into()),
            DietInstr::Set(2),
            DietInstr::Label("label_0".into()),
            DietInstr::Set(1),
            DietInstr::Jez("label_1".into()),
            DietInstr::Set(3),
            DietInstr::Label("label_1".into()),
            DietInstr::Set(1),
        ]);

        assert_eq!(
            compiler.delabel().0,
            vec![
                MachineInstr::Set(0),
                MachineInstr::Jez(3), // --+
                MachineInstr::Set(2), //   |
                MachineInstr::Set(1), // <-+
                MachineInstr::Jez(6), // --+
                MachineInstr::Set(3), //   |
                MachineInstr::Set(1), // <-+
            ]
        );
    }

    #[test]
    #[should_panic]
    fn test_duplicate_label_panic() {
        let compiler = Compiler(vec![
            DietInstr::Label("label_0".into()),
            DietInstr::Label("label_0".into()),
        ]);
        compiler.delabel();
    }

    #[test]
    #[should_panic]
    fn test_unknown_label_panic() {
        let compiler = Compiler(vec![DietInstr::Jez("label_0".into())]);
        compiler.delabel();
    }
}
