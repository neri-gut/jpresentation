// Tipos TypeScript para el plugin JWPUB Parser
// Usados en Pinia stores y componentes de Vue 3

export interface MeetingMediaItem {
  id: string;
  title: string;
  type: 'image' | 'video' | 'audio';
  filePath: string;
  localUrl: string;
  width?: number;
  height?: number;
  caption?: string;
}

export interface StudentPart {
  partIndex: number;
  title: string;
  assignedTimeMinutes: number;
  partType: 'reading' | 'initial_call' | 'return_visit' | 'bible_study' | 'talk' | 'other';
  studentName?: string;
  assistantName?: string;
}

export interface MWBWeeklySchedule {
  weekDate: string;
  weekDateFormatted: string;
  weeklyBibleReading: string;
  openingSong: number;
  middleSong: number;
  concludingSong: number;
  treasures: {
    talkTitle: string;
    talkTimeMinutes: number;
    spiritualGemsTitle: string;
    bibleReading: string;
  };
  fieldMinistryParts: StudentPart[];
  livingAsChristians: {
    parts: Array<{ title: string; timeMinutes: number; content?: string }>;
    congregationBibleStudy: { title: string; timeMinutes: number };
  };
  mediaItems: MeetingMediaItem[];
}

export interface WatchtowerWeeklySchedule {
  studyDate: string;
  articleTitle: string;
  openingSong: number;
  concludingSong: number;
  paragraphs: Array<{ pid: number; text: string; question?: string }>;
  mediaItems: MeetingMediaItem[];
}

export interface JwpubImportResult {
  publicationSymbol: string;
  year: number;
  issue: string;
  title: string;
  language: string;
  mwbWeeks?: MWBWeeklySchedule[];
  watchtowerWeeks?: WatchtowerWeeklySchedule[];
}
