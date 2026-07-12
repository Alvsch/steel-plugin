--!strict
export type DataStore = {
	SetAsync: (self: DataStore, key: any, value: any) -> (),
	GetAsync: (self: DataStore, key: any) -> any,
	UpdateAsync: (self: DataStore, key: any, update: (any) -> any) -> (),
	RemoveAsync: (self: DataStore, key: any) -> any,
}
