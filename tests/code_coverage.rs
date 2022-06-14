use fuel_tx::ScriptExecutionResult;
use std::sync::{Arc, Mutex};

use fuel_vm::consts::*;
use fuel_vm::prelude::*;
use fuel_vm::profiler::{InstructionLocation, ProfileReceiver, ProfilingData};

const HALF_WORD_SIZE: u64 = 4;

#[test]
fn code_coverage() {
    let gas_price = 1;
    let gas_limit = 1_000;
    let byte_price = 0;
    let maturity = 0;

    // Deploy contract with loops
    let reg_a = 0x20;

    let script_code: Vec<Opcode> = vec![
        Opcode::JNEI(REG_ZERO, REG_ONE, 2),  // Skip next
        Opcode::XOR(reg_a, reg_a, reg_a),    // Skipped
        Opcode::JNEI(REG_ZERO, REG_ZERO, 2), // Do not skip
        Opcode::XOR(reg_a, reg_a, reg_a),    // Executed
        Opcode::RET(REG_ONE),
    ];

    let tx_script = Transaction::script(
        gas_price,
        gas_limit,
        byte_price,
        maturity,
        script_code.into_iter().collect(),
        vec![],
        vec![],
        vec![],
        vec![],
    );

    #[derive(Clone, Default)]
    struct ProfilingOutput {
        data: Arc<Mutex<Option<ProfilingData>>>,
    }

    impl ProfileReceiver for ProfilingOutput {
        fn on_transaction(&mut self, _state: &Result<ProgramState, InterpreterError>, data: &ProfilingData) {
            let mut guard = self.data.lock().unwrap();
            *guard = Some(data.clone());
        }
    }

    let output = ProfilingOutput::default();

    let mut client = MemoryClient::from_txtor(
        Interpreter::with_memory_storage()
            .with_profiler(output.clone())
            .build()
            .into(),
    );

    let receipts = client.transact(tx_script);

    if let Some(Receipt::ScriptResult { result, .. }) = receipts.last() {
        assert!(matches!(result, ScriptExecutionResult::Success));
    } else {
        panic!("Missing result receipt");
    }

    let guard = output.data.lock().unwrap();
    let case = guard.as_ref().unwrap().clone();

    let mut items: Vec<_> = case.coverage().iter().collect();
    items.sort();

    let expect = vec![0, 2, 3, 4];

    assert_eq!(items.len(), expect.len());

    for (item, expect) in items.into_iter().zip(expect.into_iter()) {
        assert_eq!(*item, InstructionLocation::new(None, expect * HALF_WORD_SIZE));
    }
}
