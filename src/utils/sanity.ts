import {useSanityClient} from '@sanity/astro'
import type {PortableTextBlock} from '@portabletext/types'
import type {ImageAsset, Slug} from '@sanity/types'
import groq from 'groq'

export async function getWishes(): Promise<Wish[]> {
    return await useSanityClient().fetch(
        groq`*[_type == "wish" && defined(slug.current)] | order(orderRank)`
    )
}

export async function getWish(slug: string): Promise<Wish> {
    return await useSanityClient().fetch(
        groq`*[_type == "wish" && slug.current == $slug][0]`,
        {
            slug
        }
    )
}

export interface Wish {
    _type: 'wish'
    _createdAt: string
    title?: string
    slug: Slug
    excerpt?: string
    url?: string,
    mainImage?: ImageAsset
    body: PortableTextBlock[]
}

export type PostRank = number
