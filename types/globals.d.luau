--!strict
export type Game = {
	Store: DataStore,
}

declare game: Game
declare signal: Signal<string>

declare error: (message: string) -> ()
declare warn: (message: string) -> ()
declare info: (message: string) -> ()
declare debug: (message: string) -> ()
