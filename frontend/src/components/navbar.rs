use yew::prelude::*;
use yew_router::prelude::*;
use crate::Route;

#[function_component(Navbar)]
pub fn navbar() -> Html {
    html! {
        <nav class="bg-white shadow-lg">
            <div class="container mx-auto px-4">
                <div class="flex justify-between items-center py-4">
                    <Link<Route> to={Route::Home} classes="text-2xl font-bold text-purple-600">
                        {"🎵 Spotify Neo4j"}
                    </Link<Route>>
                    
                    <div class="flex space-x-6">
                        <Link<Route> 
                            to={Route::Home} 
                            classes="text-gray-700 hover:text-purple-600 px-3 py-2 rounded-md text-sm font-medium transition-colors"
                        >
                            {"Home"}
                        </Link<Route>>
                        
                        <Link<Route> 
                            to={Route::Artists} 
                            classes="text-gray-700 hover:text-purple-600 px-3 py-2 rounded-md text-sm font-medium transition-colors"
                        >
                            {"Artists"}
                        </Link<Route>>
                        
                        <Link<Route> 
                            to={Route::Tracks} 
                            classes="text-gray-700 hover:text-purple-600 px-3 py-2 rounded-md text-sm font-medium transition-colors"
                        >
                            {"Tracks"}
                        </Link<Route>>
                        
                        <Link<Route> 
                            to={Route::Recommendations} 
                            classes="text-gray-700 hover:text-purple-600 px-3 py-2 rounded-md text-sm font-medium transition-colors"
                        >
                            {"Recommendations"}
                        </Link<Route>>
                        
                        <Link<Route> 
                            to={Route::Playlists} 
                            classes="text-gray-700 hover:text-purple-600 px-3 py-2 rounded-md text-sm font-medium transition-colors"
                        >
                            {"Playlists"}
                        </Link<Route>>
                    </div>
                </div>
            </div>
        </nav>
    }
}
