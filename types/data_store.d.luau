--!strict
export type DataStore = {
	SetAsync: (self: DataStore, key: string, value: any) -> (),
	GetAsync: (self: DataStore, key: string) -> any,
	UpdateAsync: (self: DataStore, key: string, update: (old: any) -> any) -> (),
	RemoveAsync: (self: DataStore, key: string) -> any,
}
