export interface SessionUser {
  id: string;
  email: string;
  username: string;
  is_admin: boolean;
}

const TOKEN_KEY = 'token';
const USER_KEY = 'user';

export function getAccessToken(): string | null {
  return localStorage.getItem(TOKEN_KEY);
}

export function setAccessToken(token: string): void {
  localStorage.setItem(TOKEN_KEY, token);
}

export function getStoredUser(): SessionUser | null {
  const saved = localStorage.getItem(USER_KEY);
  if (!saved) return null;

  try {
    return JSON.parse(saved) as SessionUser;
  } catch {
    return null;
  }
}

export function setStoredUser(user: SessionUser): void {
  localStorage.setItem(USER_KEY, JSON.stringify(user));
}

export function clearSession(): void {
  localStorage.removeItem(TOKEN_KEY);
  localStorage.removeItem(USER_KEY);
}

function decodeLegacyJwtUser(token: string): SessionUser | null {
  try {
    const payload = JSON.parse(atob(token.split('.')[1]));
    return {
      id: payload.sub,
      email: payload.email,
      username: payload.username,
      is_admin: payload.admin,
    };
  } catch {
    return null;
  }
}

export function consumeOAuthCallbackSession(): { token: string; user: SessionUser | null } | null {
  const hash = window.location.hash;
  if (!hash.includes('token=')) return null;

  const query = hash.split('?')[1];
  if (!query) return null;

  const params = new URLSearchParams(query);
  const token = params.get('token');
  if (!token) return null;

  const user = decodeLegacyJwtUser(token);
  window.location.hash = '';
  return { token, user };
}
