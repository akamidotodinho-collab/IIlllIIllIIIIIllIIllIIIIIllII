// Schemas compartilhados para o ARKIVE Desktop
export interface Document {
  id: string;
  name: string;
  size: number;
  type: string;
  uploadDate: string;
  isActive: boolean;
  category: string;
}

export interface User {
  id: string;
  username: string;
  email?: string;
}

export interface Activity {
  id: string;
  action: string;
  document: string;
  timestamp: string;
  user: string;
}

export interface Stats {
  total_documents: number;
  uploads_today: number;
  total_size: string;
  active_documents: number;
}