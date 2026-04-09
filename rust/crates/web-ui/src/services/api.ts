import axios from 'axios';
import type { Project, AIModel, CreateProjectRequest } from '../types';

const API_BASE_URL = '/api';

export const api = {
  getProjects: (): Promise<{ data: Project[] }> => 
    axios.get(`${API_BASE_URL}/projects`),
  
  createProject: (project: CreateProjectRequest): Promise<void> => 
    axios.post(`${API_BASE_URL}/projects`, project),
  
  getAIModels: (): Promise<{ data: AIModel[] }> => 
    axios.get(`${API_BASE_URL}/ai-models`),
  
  healthCheck: (): Promise<void> => 
    axios.get(`${API_BASE_URL}/health`),
};
