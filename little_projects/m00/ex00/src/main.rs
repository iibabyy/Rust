/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   main.rs                                            :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: ibaby <ibaby@student.42.fr>                +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/09/19 17:33:23 by ibaby             #+#    #+#             */
/*   Updated: 2024/09/19 18:00:36 by ibaby            ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

use std::env;

fn main()
{
    let	mut args: Vec<String> = env::args().collect();
	
	if args.len() <= 1 {
		println!("* LOUD AND UNBEARABLE FEEDBACK NOISE *");
	} else {
		args.remove(0);
		for c in args {
			print!("{} ", c.to_uppercase());
		}
	}
}
