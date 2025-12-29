import type { PortableTextBlock } from '@portabletext/types';
import type { ImageAsset, Slug } from '@sanity/types';
import groq from 'groq';
import { DateTime } from 'luxon';
import { LocalizedDateTime } from '../utils/date.ts';
import type { SanityClient } from '@sanity/client';

export async function getWishes(client: SanityClient): Promise<Wish[]> {
  return await client.fetch(
    groq`*[_type == "wish" && defined(slug.current)] | order(orderRank)`,
  );
}

export async function getLastModified(
  client: SanityClient,
): Promise<string | null> {
  const lastModifiedISO = await client.fetch(
    groq`*[_type == "wish"] | order(_updatedAt desc)[0]._updatedAt`,
  );
  if (lastModifiedISO == undefined) {
    return null;
  }
  const lastModifiedDate = DateTime.fromISO(lastModifiedISO);
  return lastModifiedDate.toLocaleString({
    day: 'numeric',
    month: 'long',
    // show year if not within current year
    year: lastModifiedDate.hasSame(LocalizedDateTime.now(), 'year')
      ? undefined
      : 'numeric',
  });
}

export interface Wish {
  _type: 'wish';
  _createdAt: string;
  title?: string;
  slug: Slug;
  excerpt?: string;
  url?: string;
  mainImage?: ImageAsset;
  body: PortableTextBlock[];
}

export type PostRank = number;
