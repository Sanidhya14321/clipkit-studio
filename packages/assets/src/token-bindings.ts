import {getProperty,setProperty} from './property-path';
import type {Item} from '../../core/src/index';
import type {DesignTokens} from '../../core/src/brief';
export function applyTokenBindings(item:Item,tokens?:DesignTokens){let props=item.props;if(tokens)for(const [property,token]of Object.entries(item.tokenBindings??{})){const value=getProperty(tokens,token);if(value!==undefined)props=setProperty(props,property,value)}return props;}
