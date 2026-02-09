export interface ServerInfo {
  id: string;
  playing: number;
  maxPlayers: number;
  ping?: number;
  fps?: number;
}

export interface GameInfo {
  universeId: number;
  placeId: number;
  name: string;
  description: string;
  creatorName: string;
  playing: number;
  visits: number;
  thumbnail?: string;
}

export interface BrowsedGame {
  universeId: number;
  placeId: number;
  name: string;
  playerCount: number;
  thumbnail?: string;
}

export interface FavoriteGame {
  placeId: number;
  universeId: number;
  name: string;
  thumbnail?: string;
}
