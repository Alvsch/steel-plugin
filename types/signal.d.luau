--!strict
declare Signal: {
	new: () -> UserSignal<any>,
}

export type Signal<T> = {
	Connect: (self: Signal<T>, (T) -> ()) -> Connection,
	Once: (self: Signal<T>, (T) -> ()) -> Connection,
	Wait: (self: Signal<T>) -> T,
}

export type UserSignal<T> = Signal<T> & {
	Emit: (self: Signal<T>, T) -> ()
}

export type Connection = {
	Disconnect: () -> (),
	Connected: boolean,
}
