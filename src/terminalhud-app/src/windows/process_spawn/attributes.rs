use crate::windows::native::last;
use std::{io, marker::PhantomData, ptr};
use windows_sys::Win32::{
    Foundation::HANDLE,
    System::{Memory::*, Threading::*},
};
/// Attribute values and inherited handles outlive this list and CreateProcess.
pub(super) struct Attributes<'a> {
    heap: HANDLE,
    pub pointer: LPPROC_THREAD_ATTRIBUTE_LIST,
    initialized: bool,
    handles: PhantomData<&'a [HANDLE]>,
}
impl<'a> Attributes<'a> {
    pub(super) fn new(handles: &'a [HANDLE]) -> io::Result<Self> {
        let mut bytes = 0;
        unsafe {
            InitializeProcThreadAttributeList(ptr::null_mut(), 1, 0, &mut bytes);
        }
        if bytes == 0 {
            return Err(last("InitializeProcThreadAttributeList size"));
        }
        let heap = unsafe { GetProcessHeap() };
        if heap.is_null() {
            return Err(last("GetProcessHeap"));
        }
        let pointer = unsafe { HeapAlloc(heap, 0, bytes) };
        if pointer.is_null() {
            return Err(io::Error::other("process attribute allocation failed"));
        }
        let mut attributes = Self {
            heap,
            pointer,
            initialized: false,
            handles: PhantomData,
        };
        if unsafe { InitializeProcThreadAttributeList(pointer, 1, 0, &mut bytes) } == 0 {
            return Err(last("InitializeProcThreadAttributeList"));
        }
        attributes.initialized = true;
        // Real inheritable duplicates only; no pseudo handles in this list.
        if unsafe {
            UpdateProcThreadAttribute(
                pointer,
                0,
                PROC_THREAD_ATTRIBUTE_HANDLE_LIST as usize,
                handles.as_ptr().cast(),
                std::mem::size_of_val(handles),
                ptr::null_mut(),
                ptr::null_mut(),
            )
        } == 0
        {
            return Err(last("UpdateProcThreadAttribute HANDLE_LIST"));
        }
        Ok(attributes)
    }
}
impl Drop for Attributes<'_> {
    fn drop(&mut self) {
        unsafe {
            if self.initialized {
                DeleteProcThreadAttributeList(self.pointer);
            }
            HeapFree(self.heap, 0, self.pointer);
        }
    }
}
