# Conventions
General conventions for `steel-host` and `steel-plugin-sdk`.

## Memory conventions

### Allocation/Deallocation
| Case                            | Allocator | Deallocator               |
|---------------------------------|-----------|---------------------------|
| host→module, parameter, scratch | nobody    | nobody                    |
| host→module, parameter, heap    | host      | host after call returns   |
| host→module, return value, heap | host      | module deallocs           |
| module→host, parameter, heap    | module    | module after call returns |
| module→host, return value, heap | module    | host calls `dealloc`      |

### Scratch Space
May be used by the host for temporary allocations that pass over to the module through parameters.
The scratch should never be deallocated.

## Host functions
Host functions may not call any module function except `alloc` and `dealloc`.
This could cause reentrancy which may cause deadlocks and/or destroy the scratch.
