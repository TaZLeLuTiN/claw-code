import React from 'react';
import { Typography } from 'antd';

const { Paragraph } = Typography;

interface FormattedResponseProps {
  content: string;
}

const FormattedResponse: React.FC<FormattedResponseProps> = ({ content }) => {
  const formatContent = (text: string) => {
    // Nettoyage initial du texte
    const cleanedText = text
      .replace(/^# (.*$)/gim, 'TITLE_H1: $1')
      .replace(/^## (.*$)/gim, 'TITLE_H2: $1') 
      .replace(/^### (.*$)/gim, 'TITLE_H3: $1')
      .replace(/^[*\-] (.*$)/gim, '• $1');

    // Séparation par paragraphes
    const sections = cleanedText.split('\n\n').filter(section => section.trim());
    
    return sections.map((section, index) => {
      // Titres
      if (section.startsWith('TITLE_H1: ')) {
        return (
          <h1 key={index} style={{ fontSize: '1.5em', fontWeight: 'bold', margin: '16px 0 8px 0' }}>
            {section.replace('TITLE_H1: ', '')}
          </h1>
        );
      }
      if (section.startsWith('TITLE_H2: ')) {
        return (
          <h2 key={index} style={{ fontSize: '1.3em', fontWeight: 'bold', margin: '14px 0 6px 0' }}>
            {section.replace('TITLE_H2: ', '')}
          </h2>
        );
      }
      if (section.startsWith('TITLE_H3: ')) {
        return (
          <h3 key={index} style={{ fontSize: '1.1em', fontWeight: 'bold', margin: '12px 0 4px 0' }}>
            {section.replace('TITLE_H3: ', '')}
          </h3>
        );
      }
      
      // Listes
      if (section.includes('• ')) {
        const items = section.split('\n').filter(item => item.trim() && item.includes('• '));
        return (
          <ul key={index} style={{ paddingLeft: '20px', margin: '8px 0' }}>
            {items.map((item, i) => (
              <li key={i}>{item.replace('• ', '').trim()}</li>
            ))}
          </ul>
        );
      }
      
      // Paragraphes normaux
      return (
        <Paragraph key={index} style={{ margin: '8px 0', whiteSpace: 'pre-wrap' }}>
          {section}
        </Paragraph>
      );
    });
  };

  return (
    <div style={{ textAlign: 'left' }}>
      {formatContent(content)}
    </div>
  );
};

export default FormattedResponse;
