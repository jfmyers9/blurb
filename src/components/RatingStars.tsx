import { useState } from "react";

interface RatingStarsProps {
  rating: number | null;
  onRate: (score: number) => void;
  size?: "sm" | "md";
}

export default function RatingStars({
  rating,
  onRate,
  size = "md",
}: RatingStarsProps) {
  const [hovered, setHovered] = useState<number | null>(null);
  const display = hovered ?? rating ?? 0;
  const starSize = size === "sm" ? "w-4 h-4" : "w-6 h-6";

  return (
    <div className="flex gap-0.5" onMouseLeave={() => setHovered(null)}>
      {[1, 2, 3, 4, 5].map((star) => (
        <button
          key={star}
          type="button"
          className="focus:outline-none"
          onMouseEnter={() => setHovered(star)}
          onClick={() => onRate(star)}
        >
          <svg
            className={`${starSize} transition-colors ${
              star <= display
                ? "text-amber-400 fill-amber-400"
                : "text-gray-300 dark:text-gray-600 fill-current"
            }`}
            viewBox="0 0 20 20"
            fill="currentColor"
          >
            <path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z" />
          </svg>
        </button>
      ))}
    </div>
  );
}
