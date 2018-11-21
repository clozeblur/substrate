// Copyright 2018 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate. If not, see <http://www.gnu.org/licenses/>.

use codec::Compact;
use runtime_support::StorageMap;
use {Schedule, Trait, CodeStorage, PrestineCode};

mod prepare;

#[derive(Clone, Encode, Decode)]
pub struct MemoryDefinition {
	#[codec(compact)]
	pub initial: u32,
	#[codec(compact)]
	pub maximum: u32,
}

#[derive(Clone, Encode, Decode)]
pub struct InstrumentedWasmModule {
	/// Version of the schedule with which the code was instrumented.
	#[codec(compact)]
	schedule_version: u32,
	pub memory_def: MemoryDefinition,
	/// Code instrumented with the latest schedule.
	pub code: Vec<u8>,
}

pub fn save<T: Trait>(
	original_code: Vec<u8>,
	schedule: &Schedule<T::Gas>,
) -> Result<(), &'static str> {
	// TODO: let code_hash = T::Hashing::hash(&original_code);
	let code_hash = T::CodeHash::default();

	// The first time instrumentation is on the user. However, consequent reinstrumentation
	// due to the schedule changes is on governance system.
	let instrumented_module = prepare::prepare_contract::<T, _>(
		&original_code,
		schedule,
		|_, _| true, // TODO: Use real validation function.
	)?;

	// TODO: validate the code. If the code is not valid, then don't store it.

	<CodeStorage<T>>::insert(code_hash, instrumented_module);
	<PrestineCode<T>>::insert(code_hash, original_code);

	panic!()
}

pub fn load<T: Trait>(code_hash: &T::CodeHash, schedule: &Schedule<T::Gas>,) -> Result<InstrumentedWasmModule, &'static str> {
	let instrumented_module = <CodeStorage<T>>::get(code_hash).ok_or_else(|| "code is not found")?;

	if instrumented_module.schedule_version < schedule.version {
		let original_code = <PrestineCode<T>>::get(code_hash).ok_or_else(|| "prestine code is not found")?;

		let instrumented_module = prepare::prepare_contract::<T, _>(
			&original_code,
			schedule,
			|_, _| true, // TODO: Use real validation function.
		)?;

		<CodeStorage<T>>::insert(code_hash, instrumented_module.clone());

		Ok(instrumented_module)
	} else {
		Ok(instrumented_module)
	}
}