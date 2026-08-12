import React from 'react';
import { DotLottiePlayer } from '@dotlottie/react-player';

export type MascotStatus = 'IDLE' | 'FOCUS' | 'BREAK' | 'SUCCESS';

export const Mascot: React.FC<{ status: MascotStatus }> = ({ status }) => {
  const getAnimationPath = () => {
    switch (status) {
      case 'FOCUS':
        return '/assets/focus.lottie';
      case 'BREAK':
        return '/assets/break.lottie';
      case 'SUCCESS':
        return '/assets/success.lottie';
      case 'IDLE':
      default:
        return '/assets/idle.lottie';
    }
  };

  return (
    <div className="w-16 h-16 pointer-events-none drop-shadow-md">
      <DotLottiePlayer
        src={getAnimationPath()}
        autoplay
        loop
      />
    </div>
  );
};

export default Mascot;
