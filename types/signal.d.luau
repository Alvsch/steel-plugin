--!strict
export type Signal<T> = {
	Connect: (self: Signal<T>, (T) -> ()) -> Connection,
	Once: (self: Signal<T>, (T) -> ()) -> Connection,
	Wait: (self: Signal<T>) -> T,
}

export type Connection = {
	Disconnect: () -> (),
	Connected: boolean,
}
