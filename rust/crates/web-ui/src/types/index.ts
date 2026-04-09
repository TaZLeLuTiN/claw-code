export interface Project {
  name: string;
  path: string;
  language: string;
  framework: string;
  created_at: string;
}

export interface AIModel {
  name: string;
  provider: string;
  capabilities: string[];
}

export interface CreateProjectRequest {
  name: string;
  path: string;
  language: string;
}
